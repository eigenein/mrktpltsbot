use anyhow::Context;
use sqlx::{FromRow, SqliteConnection};

use crate::prelude::*;

/// User's search query.
#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct SearchQuery {
    pub text: String,

    /// [SeaHash][1] of a search query.
    ///
    /// Used instead of the text where the payload size is limited (e.g. in `/start` payload).
    ///
    /// [1]: https://docs.rs/seahash/latest/seahash/
    pub hash: i64,
}

impl From<String> for SearchQuery {
    #[expect(clippy::cast_possible_wrap)]
    fn from(text: String) -> Self {
        Self { hash: seahash::hash(text.as_bytes()) as i64, text }
    }
}

pub struct SearchQueries<'a>(pub &'a mut SqliteConnection);

impl<'a> SearchQueries<'a> {
    #[instrument(skip_all, fields(text = query.text, hash = query.hash))]
    pub async fn upsert(&mut self, query: &SearchQuery) -> Result {
        // language=sql
        const QUERY: &str = "INSERT INTO search_queries (hash, text) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET text = ?2";
        sqlx::query(QUERY)
            .bind(query.hash)
            .bind(&query.text)
            .execute(&mut *self.0)
            .await
            .with_context(|| format!("failed to upsert the search query `{}`", query.text))?;

        Ok(())
    }

    #[instrument(skip_all, fields(hash = hash))]
    pub async fn fetch_text(&mut self, hash: i64) -> Result<String> {
        // language=sql
        const QUERY: &str = "SELECT text FROM search_queries WHERE hash = ?1";

        sqlx::query_scalar(QUERY)
            .bind(hash)
            .fetch_one(&mut *self.0)
            .await
            .with_context(|| format!("failed to fetch the query text for hash `{hash}`"))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::db::Db;

    #[tokio::test]
    async fn search_query_ok() -> Result {
        let db = Db::try_new(Path::new(":memory:")).await?;
        let mut connection = db.connection().await;
        let mut search_queries = SearchQueries(&mut connection);

        let query = SearchQuery::from("test".to_string());
        search_queries.upsert(&query).await?;
        search_queries.upsert(&query).await?; // verify conflicts

        assert_eq!(search_queries.fetch_text(query.hash).await?, query.text);

        Ok(())
    }
}
