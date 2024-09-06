use std::path::Path;

use anyhow::Context;
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
use uuid::Uuid;

static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Clone)]
pub struct Db(SqlitePool);

impl Db {
    pub fn new_id() -> String {
        bs58::encode(Uuid::new_v4().as_bytes()).into_string()
    }

    pub async fn new(path: &Path) -> anyhow::Result<Self> {
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
}
