use prost::Message;
use sqlx::SqliteConnection;

use crate::prelude::*;

pub struct KeyValues<'a>(pub &'a mut SqliteConnection);

impl KeyValues<'_> {
    #[instrument(
        name = "💾 Upserting key-value…",
        level = Level::DEBUG,
        skip_all,
        fields(key = M::KEY),
    )]
    pub async fn upsert<M: KeyedMessage>(&mut self, value: &M) -> Result {
        // language=sql
        const QUERY: &str = "
            INSERT INTO key_values (key, value) VALUES (?1, ?2)
            ON CONFLICT DO UPDATE SET value = ?2
        ";

        sqlx::query(QUERY)
            .bind(M::KEY)
            .bind(value.encode_to_vec())
            .execute(&mut *self.0)
            .await
            .context("failed to upsert the subscription")?;

        Ok(())
    }

    #[instrument(
        name = "💾 Fetching value…",
        level = Level::DEBUG,
        skip_all,
        fields(key = V::KEY),
    )]
    pub async fn fetch<V: Default + KeyedMessage>(&mut self) -> Result<Option<V>> {
        // language=sql
        const QUERY: &str = "SELECT value FROM key_values WHERE key = ?1";

        let value: Option<Vec<u8>> = sqlx::query_scalar(QUERY)
            .bind(V::KEY)
            .fetch_optional(&mut *self.0)
            .await
            .with_context(|| format!("failed to fetch the value for key `{}`", V::KEY))?;
        value.map_or_else(
            || Ok(None),
            |value| V::decode(value.as_slice()).context("failed to decode the value").map(Some),
        )
    }
}

pub trait KeyedMessage: Message {
    const KEY: &'static str;
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::db::Db;

    #[tokio::test]
    async fn crud_ok() -> Result {
        #[derive(PartialEq, Eq, Message)]
        struct Value {
            #[prost(tag = "1", int32)]
            pub foo: i32,
        }

        impl KeyedMessage for Value {
            const KEY: &'static str = "test";
        }

        let db = Db::try_new(Path::new(":memory:")).await?;
        let mut connection = db.connection().await;
        let mut key_values = KeyValues(&mut connection);

        assert!(key_values.fetch::<Value>().await?.is_none());

        let value = Value { foo: 42 };
        key_values.upsert(&value).await?;
        assert_eq!(key_values.fetch().await?, Some(value));

        Ok(())
    }
}
