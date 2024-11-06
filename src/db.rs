pub mod query_hash;
pub mod search_query;
pub mod subscription;

use std::path::Path;

use anyhow::Context;
use futures::{Stream, StreamExt, TryStreamExt, stream};
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

    /// Turns the database into the infinite stream of the active subscriptions.
    pub fn into_active_subscriptions(
        self,
    ) -> impl Stream<Item = Result<(Subscription, SearchQuery)>> {
        stream::try_unfold(self, move |this| async {
            Ok::<_, Error>(Some((this.active_subscriptions().await?, this)))
        })
        .map_ok(|entries| stream::iter(entries).map(Ok))
        .try_flatten()
    }

    async fn active_subscriptions(&self) -> Result<Vec<(Subscription, SearchQuery)>> {
        // language=sql
        const QUERY: &str = r"
            SELECT search_queries.*, subscriptions.* FROM subscriptions
            JOIN search_queries ON search_queries.hash = subscriptions.query_hash
        ";

        sqlx::query(QUERY)
            .fetch(&mut *self.connection().await)
            .and_then(|row| async move {
                Ok((Subscription::from_row(&row)?, SearchQuery::from_row(&row)?))
            })
            .map_err(Error::from)
            .try_collect()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{search_query::SearchQueries, subscription::Subscriptions};

    #[tokio::test]
    async fn test_active_subscriptions_ok() -> Result {
        let db = Db::try_new(Path::new(":memory:")).await?;

        let search_query = SearchQuery::from("unifi".to_string());
        SearchQueries(&mut *db.connection().await)
            .upsert(&search_query)
            .await?;

        let subscription = Subscription {
            query_hash: search_query.hash,
            chat_id: 42,
        };
        Subscriptions(&mut *db.connection().await)
            .upsert(&subscription)
            .await?;

        let mut all: Vec<_> = db.active_subscriptions().await?;
        assert_eq!(all.len(), 1);

        let (actual_subscription, actual_search_query) = all.pop().unwrap();
        assert_eq!(actual_subscription, subscription);
        assert_eq!(actual_search_query, search_query);

        Ok(())
    }
}
