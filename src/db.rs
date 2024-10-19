use std::path::Path;

use anyhow::Context;
use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

use crate::{bot::query::SearchQuery, prelude::*};

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

    pub async fn insert_search_query<'a>(&self, query: &SearchQuery) -> Result {
        sqlx::query!(
            // language=sqlite
            "INSERT INTO search_queries (hash, text) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET text = ?2",
            query.hash,
            query.text
        )
        .execute(&self.0)
        .await
        .with_context(|| format!("failed to insert search query `{}`", query.text))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn insert_search_query_ok() -> Result {
        let query = SearchQuery::from("test");

        let db = Db::new(Path::new(":memory:")).await?;

        db.insert_search_query(&query).await?;

        // Second insert to verify conflicts:
        db.insert_search_query(&query).await?;

        Ok(())
    }
}
