mod item;
pub mod notification;
pub mod search_query;
pub mod subscription;

use std::path::Path;

use anyhow::Context;
use futures::{Stream, stream};
use sqlx::{
    ConnectOptions,
    FromRow,
    SqliteConnection,
    migrate::Migrator,
    sqlite::SqliteConnectOptions,
};
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    db::{search_query::SearchQuery, subscription::Subscription},
    prelude::*,
};

static MIGRATOR: Migrator = sqlx::migrate!();

#[must_use]
pub struct Db(Mutex<SqliteConnection>);

impl Db {
    #[instrument(skip_all, fields(path = ?path))]
    pub async fn try_new(path: &Path) -> Result<Self> {
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
        info!("The database is ready");
        Ok(Self(Mutex::new(connection)))
    }

    /// Lock and return the connection.
    pub async fn connection(&self) -> MutexGuard<SqliteConnection> {
        self.0.lock().await
    }

    /// Get an endless stream of subscriptions.
    pub fn subscriptions(
        &self,
    ) -> impl Stream<Item = Result<Option<(Subscription, SearchQuery)>>> + '_ {
        stream::try_unfold((self, i64::MIN), |(this, min_hash)| async move {
            let entry = this.next_subscription(min_hash).await?;
            let (next_min_hash, _) = min_hash.overflowing_add(1);
            Ok(Some((entry, (this, next_min_hash))))
        })
    }

    #[instrument(skip_all, fields(min_hash = min_hash))]
    async fn next_subscription(
        &self,
        min_hash: i64,
    ) -> Result<Option<(Subscription, SearchQuery)>> {
        // language=sql
        const QUERY: &str = r"
            SELECT search_queries.*, subscriptions.* FROM subscriptions
            JOIN search_queries ON search_queries.hash = subscriptions.query_hash
            WHERE subscriptions.query_hash >= ?1
        ";

        let row = sqlx::query(QUERY)
            .bind(min_hash)
            .fetch_optional(&mut *self.connection().await)
            .await
            .with_context(|| format!("failed to fetch a subscription starting at {min_hash}"))?;
        match row {
            Some(row) => Ok(Some((
                Subscription::from_row(&row)?,
                SearchQuery::from_row(&row)?,
            ))),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;

    use futures::{StreamExt, TryStreamExt};

    use super::*;
    use crate::db::{search_query::SearchQueries, subscription::Subscriptions};

    #[tokio::test]
    async fn test_into_subscriptions_ok() -> Result {
        let db = Db::try_new(Path::new(":memory:")).await?;

        // Initial rows:
        let search_query = SearchQuery::from("unifi".to_string());
        let subscription = Subscription {
            query_hash: search_query.hash,
            chat_id: 42,
        };

        // Setting up:
        {
            let connection = &mut *db.connection().await;
            SearchQueries(connection).upsert(&search_query).await?;
            Subscriptions(connection).upsert(&subscription).await?;
        }

        // Test fetching the entry:
        let actual_entry = db.next_subscription(i64::MIN).await?.unwrap();
        let expected_entry = (subscription, search_query);
        assert_eq!(actual_entry, expected_entry);

        // Test fetching no entry above the query hash:
        assert!(
            db.next_subscription(i64::MAX).await?.is_none(),
            "the subscription should not be returned",
        );

        // Test repeated reading:
        let entries: Vec<_> = db.subscriptions().take(2).try_collect().await?;
        assert_eq!(entries[0].as_ref(), Some(&expected_entry));
        assert_eq!(entries[1].as_ref(), Some(&expected_entry));

        Ok(())
    }

    /// Test the subscription stream on an empty database.
    #[tokio::test]
    async fn test_empty_subscriptions_ok() -> Result {
        let db = Db::try_new(Path::new(":memory:")).await?;
        let mut entries = pin!(db.subscriptions());
        assert_eq!(entries.try_next().await?, Some(None));
        Ok(())
    }
}
