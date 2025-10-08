mod item;
mod key_values;
mod notification;
mod search_query;
mod subscription;

use std::{path::Path, sync::Arc};

use anyhow::Context;
use sqlx::{
    ConnectOptions,
    FromRow,
    SqliteConnection,
    migrate::Migrator,
    sqlite::SqliteConnectOptions,
};
use sqlx_sqlite::SqliteRow;
use tokio::sync::{Mutex, MutexGuard};

pub use self::{
    item::{Item, Items},
    key_values::{KeyValues, KeyedMessage},
    notification::{Notification, Notifications},
    search_query::{SearchQueries, SearchQuery},
    subscription::{Subscription, Subscriptions},
};
use crate::prelude::*;

static MIGRATOR: Migrator = sqlx::migrate!();

#[must_use]
#[derive(Clone)]
pub struct Db(Arc<Mutex<SqliteConnection>>);

impl Db {
    /// TODO: change `Path` into `AsRef<Path>`.
    #[instrument(name = "💾 Opening the database…")]
    pub async fn try_new(path: &Path) -> Result<Self> {
        let mut connection = SqliteConnectOptions::new()
            .create_if_missing(true)
            .filename(path)
            .connect()
            .await
            .with_context(|| format!("failed to open database `{}`", path.display()))?;
        MIGRATOR.run(&mut connection).await.context("failed to migrate the database")?;
        info!("✅ The database is ready");
        Ok(Self(Arc::new(Mutex::new(connection))))
    }

    /// Lock and return the connection.
    pub async fn connection(&self) -> MutexGuard<'_, SqliteConnection> {
        self.0.lock().await
    }

    pub async fn subscriptions_of(&self, chat_id: i64) -> Result<Vec<(Subscription, SearchQuery)>> {
        // language=sql
        const QUERY: &str = r"
            SELECT search_queries.*, subscriptions.* FROM subscriptions
            JOIN search_queries ON search_queries.hash = subscriptions.query_hash
            WHERE subscriptions.chat_id = ?1
            ORDER BY search_queries.text
        ";

        sqlx::query(QUERY)
            .bind(chat_id)
            .fetch_all(&mut *self.connection().await)
            .await
            .with_context(|| format!("failed to fetch subscriptions of chat #{chat_id}"))?
            .into_iter()
            .map(enriched_subscription_from_row)
            .collect()
    }

    /// Retrieve the first subscription, or `None` – if there are no subscriptions.
    pub async fn first_subscription(&self) -> Result<Option<(Subscription, SearchQuery)>> {
        // language=sql
        const QUERY: &str = r"
            SELECT search_queries.*, subscriptions.* FROM subscriptions
            JOIN search_queries ON search_queries.hash = subscriptions.query_hash
            ORDER BY subscriptions.chat_id, subscriptions.query_hash
            LIMIT 1
        ";

        sqlx::query(QUERY)
            .fetch_optional(&mut *self.connection().await)
            .await
            .context("failed to fetch the first subscription")?
            .map(enriched_subscription_from_row)
            .transpose()
    }

    /// Retrieve the next subscription, or [`None`] – if `current` is the last subscription.
    #[instrument(
        name = "💾 Fetching next subscription…",
        level = Level::DEBUG,
        skip_all,
        fields(current.query_hash = current.query_hash, current.chat_id = current.chat_id),
    )]
    pub async fn next_subscription(
        &self,
        current: &Subscription,
    ) -> Result<Option<(Subscription, SearchQuery)>> {
        // language=sql
        const QUERY: &str = r"
            SELECT search_queries.*, subscriptions.* FROM subscriptions
            JOIN search_queries ON search_queries.hash = subscriptions.query_hash
            WHERE (subscriptions.chat_id, subscriptions.query_hash) > (?1, ?2)
            ORDER BY subscriptions.chat_id, subscriptions.query_hash
            LIMIT 1
        ";
        sqlx::query(QUERY)
            .bind(current.chat_id)
            .bind(current.query_hash)
            .fetch_optional(&mut *self.connection().await)
            .await
            .context("failed to fetch the next subscription")?
            .map(enriched_subscription_from_row)
            .transpose()
    }
}

#[expect(clippy::needless_pass_by_value)]
fn enriched_subscription_from_row(row: SqliteRow) -> Result<(Subscription, SearchQuery)> {
    Ok((Subscription::from_row(&row)?, SearchQuery::from_row(&row)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{search_query::SearchQueries, subscription::Subscriptions};

    #[tokio::test]
    async fn test_fetch_subscriptions_ok() -> Result {
        let db = Db::try_new(Path::new(":memory:")).await?;

        // Search queries, ordered by the hash for convenience:
        let search_query_1 = SearchQuery::from("tado");
        let search_query_2 = SearchQuery::from("unifi");

        // Subscriptions, the ordering matches the primary key and the queries:
        let subscription_first = Subscription { chat_id: 42, query_hash: search_query_1.hash };
        let subscription_middle = Subscription { chat_id: 42, query_hash: search_query_2.hash };
        let subscription_last = Subscription { chat_id: 43, query_hash: search_query_2.hash };

        // Setting up:
        {
            let connection = &mut *db.connection().await;
            SearchQueries(connection).upsert(&search_query_1).await?;
            SearchQueries(connection).upsert(&search_query_2).await?;
            Subscriptions(connection).upsert(subscription_first).await?;
            Subscriptions(connection).upsert(subscription_middle).await?;
            Subscriptions(connection).upsert(subscription_last).await?;
        }

        // Expected value shortcuts:
        let expected_entry_first = (subscription_first, search_query_1);
        let expected_entry_middle = (subscription_middle, search_query_2.clone());

        // Test the first entry:
        assert_eq!(db.first_subscription().await?.unwrap(), expected_entry_first);

        // Test fetching no entry above the last one:
        assert!(
            db.next_subscription(&subscription_last).await?.is_none(),
            "the subscription should not be returned",
        );

        // Test filtering by chat:
        assert_eq!(
            db.subscriptions_of(subscription_first.chat_id).await?,
            &[expected_entry_first, expected_entry_middle]
        );

        Ok(())
    }

    /// Test the subscription stream on an empty database.
    #[tokio::test]
    async fn test_empty_ok() -> Result {
        let db = Db::try_new(Path::new(":memory:")).await?;
        assert!(db.first_subscription().await?.is_none());
        Ok(())
    }
}
