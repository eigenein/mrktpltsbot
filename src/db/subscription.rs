use anyhow::Context;
use futures::{Stream, TryStreamExt};
use sqlx::{FromRow, SqliteConnection};

use crate::{db::query_hash::QueryHash, prelude::*};

#[derive(Debug, Eq, PartialEq, FromRow)]
pub struct Subscription {
    pub query_hash: QueryHash,
    pub chat_id: i64,
}

pub struct Subscriptions<'a>(pub &'a mut SqliteConnection);

impl<'a> Subscriptions<'a> {
    pub async fn upsert(&mut self, subscription: &Subscription) -> Result {
        sqlx::query(
            // language=sqlite
            "INSERT INTO subscriptions (query_hash, chat_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
        )
        .bind(subscription.query_hash)
        .bind(subscription.chat_id)
        .execute(&mut *self.0)
        .await
        .context("failed to upsert the subscription")?;

        Ok(())
    }

    /// Get all subscriptions from all users.
    pub fn all(&'a mut self) -> impl Stream<Item = Result<Subscription>> + 'a {
        // language=sqlite
        sqlx::query_as("SELECT * FROM subscriptions")
            .fetch(&mut *self.0)
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::db::{
        Db,
        search_query::{SearchQueries, SearchQuery},
    };

    #[tokio::test]
    async fn upsert_subscription_ok() -> Result {
        let db = Db::new(Path::new(":memory:")).await?;
        let mut connection = db.connection().await;

        let query = SearchQuery::from("test".to_string());
        let subscription = Subscription {
            query_hash: query.hash,
            chat_id: 42,
        };

        SearchQueries(&mut connection).upsert(&query).await?;

        let subscriptions: Vec<_> = {
            let mut subscriptions = Subscriptions(&mut connection);
            subscriptions.upsert(&subscription).await?;
            subscriptions.upsert(&subscription).await?; // verify conflicts
            subscriptions.all().try_collect().await?
        };

        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0], subscription);

        Ok(())
    }
}
