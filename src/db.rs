use anyhow::Context;
use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions, SqlitePool};

static MIGRATOR: Migrator = sqlx::migrate!();

pub struct Db(SqlitePool);

impl Db {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let pool = SqlitePoolOptions::new()
            .connect(url)
            .await
            .with_context(|| format!("failed to open database `{url}`"))?;
        MIGRATOR
            .run(&pool)
            .await
            .context("failed to migrate the database")?;
        Ok(Self(pool))
    }
}
