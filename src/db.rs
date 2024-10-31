pub mod query_hash;
pub mod search_query;
pub mod subscription;

use std::path::Path;

use anyhow::Context;
use sqlx::{ConnectOptions, SqliteConnection, migrate::Migrator, sqlite::SqliteConnectOptions};
use tokio::sync::{Mutex, MutexGuard};

use crate::prelude::*;

static MIGRATOR: Migrator = sqlx::migrate!();

#[must_use]
pub struct Db(Mutex<SqliteConnection>);

impl Db {
    pub async fn new(path: &Path) -> Result<Self> {
        let mut connection = SqliteConnectOptions::new()
            .create_if_missing(true)
            .filename(path)
            .connect()
            .await
            .with_context(|| format!("failed to open database `{path:?}`"))?;
        MIGRATOR
            .run(&mut connection)
            .await
            .context("failed to migrate the database")?;
        Ok(Self(Mutex::new(connection)))
    }

    pub async fn connection(&self) -> MutexGuard<SqliteConnection> {
        self.0.lock().await
    }
}
