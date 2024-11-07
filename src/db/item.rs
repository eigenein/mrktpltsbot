use chrono::{DateTime, Utc};
use sqlx::SqliteConnection;

use crate::prelude::*;

pub struct Item {
    pub id: String,
    pub updated_at: DateTime<Utc>,
}

pub struct Items<'a>(pub &'a mut SqliteConnection);

impl<'a> Items<'a> {
    #[instrument(skip_all, fields(id = item.id, updated_at = ?item.updated_at))]
    pub async fn upsert(&mut self, item: &Item) -> Result {
        // language=sql
        const QUERY: &str = "
            INSERT INTO items (id, updated_at) VALUES (?1, ?2)
            ON CONFLICT DO UPDATE SET updated_at = ?2
        ";
        sqlx::query(QUERY)
            .bind(&item.id)
            .bind(item.updated_at)
            .execute(&mut *self.0)
            .await
            .context("failed to upsert the item")?;

        Ok(())
    }
}
