use std::path::Path;

use anyhow::Context;
use sqlx::{FromRow, SqliteConnection};

use crate::{
    db::{Db, query_hash::QueryHash},
    prelude::*,
};

/// User's search query.
#[derive(FromRow)]
pub struct SearchQuery {
    pub text: String,
    pub hash: QueryHash,
}

impl From<String> for SearchQuery {
    fn from(text: String) -> Self {
        Self {
            hash: QueryHash::from(text.as_str()),
            text,
        }
    }
}

pub struct SearchQueries<'a>(pub &'a mut SqliteConnection);

impl<'a> SearchQueries<'a> {
    pub async fn upsert(&mut self, query: &SearchQuery) -> Result {
        sqlx::query(
            // language=sqlite
            "INSERT INTO search_queries (hash, text) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET text = ?2",
        )
            .bind(query.hash)
            .bind(&query.text)
            .execute(&mut *self.0)
            .await
            .with_context(|| format!("failed to upsert the search query `{}`", query.text))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn upsert_search_query_ok() -> Result {
        let db = Db::new(Path::new(":memory:")).await?;
        let mut connection = db.connection().await;
        let mut search_queries = SearchQueries(&mut connection);

        let query = SearchQuery::from("test".to_string());
        search_queries.upsert(&query).await?;
        search_queries.upsert(&query).await?; // verify conflicts

        Ok(())
    }
}
