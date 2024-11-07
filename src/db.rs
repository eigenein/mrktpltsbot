pub mod item;
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
        MIGRATOR.run(&mut connection).await.context("failed to migrate the database")?;
        info!("The database is ready");
        Ok(Self(Mutex::new(connection)))
    }

    /// Lock and return the connection.
    pub async fn connection(&self) -> MutexGuard<SqliteConnection> {
        self.0.lock().await
    }

    /// Get an endless stream of subscriptions.
    ///
    /// - Yields [`Some`] while there are still rows in the table, and restarts when the end is reached.
    /// - Yields [`None`] if the table is empty.
    pub fn subscriptions(
        &self,
    ) -> impl Stream<Item = Result<Option<(Subscription, SearchQuery)>>> + '_ {
        stream::try_unfold((self, None), |(this, previous)| async move {
            let entry = match previous {
                None => this.first_subscription().await?,
                Some(previous) => match this.next_subscription(previous).await? {
                    Some(next) => Some(next),
                    None => this.first_subscription().await?, // reached the end, restart
                },
            };
            let next = entry.as_ref().map(|(subscription, _)| *subscription);
            Ok(Some((entry, (this, next))))
        })
    }

    #[instrument(skip_all)]
    async fn first_subscription(&self) -> Result<Option<(Subscription, SearchQuery)>> {
        // language=sql
        const QUERY: &str = r"
            SELECT search_queries.*, subscriptions.* FROM subscriptions
            JOIN search_queries ON search_queries.hash = subscriptions.query_hash
            ORDER BY subscriptions.chat_id, subscriptions.query_hash
            LIMIT 1
        ";

        let row = sqlx::query(QUERY)
            .fetch_optional(&mut *self.connection().await)
            .await
            .context("failed to fetch the first subscription")?;
        match row {
            Some(row) => Ok(Some((Subscription::from_row(&row)?, SearchQuery::from_row(&row)?))),
            None => Ok(None),
        }
    }

    #[instrument(skip_all, fields(query_hash = current.query_hash, chat_id = current.chat_id))]
    async fn next_subscription(
        &self,
        current: Subscription,
    ) -> Result<Option<(Subscription, SearchQuery)>> {
        // language=sql
        const QUERY: &str = r"
            SELECT search_queries.*, subscriptions.* FROM subscriptions
            JOIN search_queries ON search_queries.hash = subscriptions.query_hash
            WHERE (subscriptions.chat_id, subscriptions.query_hash) > (?1, ?2)
            ORDER BY subscriptions.chat_id, subscriptions.query_hash
            LIMIT 1
        ";

        let row = sqlx::query(QUERY)
            .bind(current.chat_id)
            .bind(current.query_hash)
            .fetch_optional(&mut *self.connection().await)
            .await
            .context("failed to fetch the next subscription")?;
        match row {
            Some(row) => Ok(Some((Subscription::from_row(&row)?, SearchQuery::from_row(&row)?))),
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
        let search_query_first = SearchQuery::from("tado".to_string());
        let subscription_first = Subscription { query_hash: search_query_first.hash, chat_id: 42 };
        let search_query_last = SearchQuery::from("unifi".to_string());
        let subscription_last = Subscription { query_hash: search_query_last.hash, chat_id: 42 };

        // Setting up:
        {
            let connection = &mut *db.connection().await;
            SearchQueries(connection).upsert(&search_query_first).await?;
            Subscriptions(connection).upsert(subscription_first).await?;
            SearchQueries(connection).upsert(&search_query_last).await?;
            Subscriptions(connection).upsert(subscription_last).await?;
        }

        // Test fetching the first entry:
        let actual_entry = db.first_subscription().await?.unwrap();
        let expected_entry_first = (subscription_first, search_query_first);
        let expected_entry_last = (subscription_last, search_query_last);
        assert_eq!(actual_entry, expected_entry_first);

        // Test fetching no entry above the last one:
        assert!(
            db.next_subscription(subscription_last).await?.is_none(),
            "the subscription should not be returned",
        );

        // Test repeated reading:
        let entries: Vec<_> = db.subscriptions().take(4).try_collect().await?;
        assert_eq!(entries[0].as_ref(), Some(&expected_entry_first));
        assert_eq!(entries[1].as_ref(), Some(&expected_entry_last));
        assert_eq!(entries[2].as_ref(), Some(&expected_entry_first));
        assert_eq!(entries[3].as_ref(), Some(&expected_entry_last));

        Ok(())
    }

    /// Test the subscription stream on an empty database.
    #[tokio::test]
    async fn test_empty_subscriptions_ok() -> Result {
        let db = Db::try_new(Path::new(":memory:")).await?;
        let mut entries = pin!(db.subscriptions());
        assert_eq!(entries.try_next().await?, Some(None));
        assert_eq!(entries.try_next().await?, Some(None));
        Ok(())
    }
}
