use std::path::Path;

use anyhow::Context;
use sqlx::SqliteConnection;

use crate::{db::Db, prelude::*};

/// User's search query.
pub struct SearchQuery {
    pub text: String,

    /// SeaHash-ed text.
    ///
    /// Used instead of the text where the payload size is limited.
    pub hash: u64,
}

impl From<&str> for SearchQuery {
    fn from(text: &str) -> Self {
        let text = text.trim().to_lowercase();
        Self {
            hash: seahash::hash(text.as_bytes()),
            text,
        }
    }
}

pub struct SearchQueries<'a>(pub &'a mut SqliteConnection);

impl<'a> SearchQueries<'a> {
    pub async fn upsert(&mut self, query: &SearchQuery) -> Result {
        // SQLx does not support `u64`.
        #[expect(clippy::cast_possible_wrap)]
        let hash = query.hash as i64;

        sqlx::query!(
            // language=sqlite
            "INSERT INTO search_queries (hash, text) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET text = ?2",
            hash,
            query.text
        )
            .execute(&mut *self.0)
            .await
            .with_context(|| format!("failed to insert the search query `{}`", query.text))?;

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
        let mut connection = db.connection().await;
        let mut search_queries = SearchQueries(&mut connection);
        search_queries.upsert(&query).await?;
        search_queries.upsert(&query).await?; // verify conflicts
        Ok(())
    }
}
