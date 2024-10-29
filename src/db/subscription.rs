use anyhow::Context;
use sqlx::SqliteConnection;

use crate::prelude::*;

pub struct Subscription {
    pub query_hash: u64,
    pub chat_id: i64,
}

pub struct Subscriptions<'a>(&'a mut SqliteConnection);

impl<'a> Subscriptions<'a> {
    pub async fn upsert(&mut self, subscription: &Subscription) -> Result {
        // SQLx does not support `u64`.
        #[expect(clippy::cast_possible_wrap)]
        let hash = subscription.query_hash as i64;

        sqlx::query!(
            // language=sqlite
            "INSERT INTO subscriptions (query_hash, chat_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
            hash,
            subscription.chat_id
        )
            .execute(&mut *self.0)
            .await
            .context("failed to insert the subscription")?;

        Ok(())
    }
}
