use anyhow::Context;
use sqlx::{FromRow, SqliteConnection};

use crate::prelude::*;

#[derive(Debug, Eq, PartialEq, FromRow)]
pub struct Subscription {
    pub query_hash: i64,
    pub chat_id: i64,
}

pub struct Subscriptions<'a>(pub &'a mut SqliteConnection);

impl<'a> Subscriptions<'a> {
    #[instrument(skip_all, fields(query_hash = subscription.query_hash, chat_id = subscription.chat_id))]
    pub async fn upsert(&mut self, subscription: &Subscription) -> Result {
        sqlx::query(
            // language=sql
            "INSERT INTO subscriptions (query_hash, chat_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
        )
        .bind(subscription.query_hash)
        .bind(subscription.chat_id)
        .execute(&mut *self.0)
        .await
        .context("failed to upsert the subscription")?;

        Ok(())
    }

    #[instrument(skip_all, fields(query_hash = subscription.query_hash, chat_id = subscription.chat_id))]
    pub async fn delete(&mut self, subscription: &Subscription) -> Result {
        sqlx::query(
            // language=sql
            "DELETE FROM subscriptions WHERE query_hash = ?1 AND chat_id = ?2",
        )
        .bind(subscription.query_hash)
        .bind(subscription.chat_id)
        .execute(&mut *self.0)
        .await
        .context("failed to delete the subscription")?;

        Ok(())
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
        let db = Db::try_new(Path::new(":memory:")).await?;
        let mut connection = db.connection().await;

        let query = SearchQuery::from("test".to_string());
        SearchQueries(&mut connection).upsert(&query).await?;

        let mut subscriptions = Subscriptions(&mut connection);
        let subscription = Subscription {
            query_hash: query.hash,
            chat_id: 42,
        };

        subscriptions.upsert(&subscription).await?;
        subscriptions.upsert(&subscription).await?; // verify conflicts

        Ok(())
    }
}
