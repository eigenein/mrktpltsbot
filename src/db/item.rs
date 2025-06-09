use chrono::{DateTime, Utc};
use sqlx::SqliteConnection;

use crate::prelude::*;

#[derive(Copy, Clone)]
pub struct Item<'a> {
    pub id: &'a str,
    pub updated_at: DateTime<Utc>,
}

pub struct Items<'a>(pub &'a mut SqliteConnection);

impl Items<'_> {
    pub async fn upsert(&mut self, item: Item<'_>) -> Result {
        // language=sql
        const QUERY: &str = "
            INSERT INTO items (id, updated_at) VALUES (?1, ?2)
            ON CONFLICT DO UPDATE SET updated_at = ?2
        ";

        debug!(
            "💾 Upserting item…",
            id = item.id.to_string(),
            updated_at = item.updated_at.timestamp(),
        );
        sqlx::query(QUERY)
            .bind(item.id)
            .bind(item.updated_at)
            .execute(&mut *self.0)
            .await
            .with_context(|| format!("failed to upsert the item #{}", item.id))?;

        Ok(())
    }
}
