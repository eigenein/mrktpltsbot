use anyhow::Context;

use crate::db::{Db, Insert};

pub struct Subscription {
    pub query_hash: u64,
    pub chat_id: i64,
}

impl Insert<Subscription> for Db {
    async fn insert(&self, subscription: &Subscription) -> crate::prelude::Result {
        // SQLx does not support `u64`.
        #[expect(clippy::cast_possible_wrap)]
        let hash = subscription.query_hash as i64;

        sqlx::query!(
            // language=sqlite
            "INSERT INTO subscriptions (query_hash, chat_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
            hash,
            subscription.chat_id
        )
            .execute(&self.0)
            .await
            .context("failed to insert the subscription")?;

        Ok(())
    }
}
