use std::path::Path;

use anyhow::Context;
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};

use crate::prelude::*;

static MIGRATOR: Migrator = sqlx::migrate!();

#[must_use]
pub struct Db(SqlitePool);

impl Db {
    pub async fn new(path: &Path) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .create_if_missing(true)
            .filename(path)
            .journal_mode(SqliteJournalMode::Wal);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .with_context(|| format!("failed to open database `{path:?}`"))?;
        MIGRATOR
            .run(&pool)
            .await
            .context("failed to migrate the database")?;
        Ok(Self(pool))
    }

    pub async fn insert_search_query(&self, text: &str) -> Result<i64> {
        #[allow(clippy::cast_possible_wrap)]
        let hash = seahash::hash(text.as_bytes()) as i64;

        sqlx::query!(
            // language=sqlite
            "INSERT INTO search_queries (hash, text) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET text = ?2",
            hash,
            text
        )
        .execute(&self.0)
        .await
        .with_context(|| format!("failed to insert search query `{text}`"))?;

        Ok(hash)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn insert_search_query_ok() -> Result {
        let hash = Db::new(Path::new(":memory:"))
            .await?
            .insert_search_query("test")
            .await?;
        assert_eq!(hash, 6214865450970028004);

        // Second insert to verify conflicts:
        Db::new(Path::new(":memory:"))
            .await?
            .insert_search_query("test")
            .await?;

        Ok(())
    }
}
