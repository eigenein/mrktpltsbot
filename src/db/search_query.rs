use anyhow::Context;

use crate::db::{Db, Insert};

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

impl Insert<SearchQuery> for Db {
    async fn insert(&self, query: &SearchQuery) -> crate::prelude::Result {
        // SQLx does not support `u64`.
        #[expect(clippy::cast_possible_wrap)]
        let hash = query.hash as i64;

        sqlx::query!(
            // language=sqlite
            "INSERT INTO search_queries (hash, text) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET text = ?2",
            hash,
            query.text
        )
            .execute(&self.0)
            .await
            .with_context(|| format!("failed to insert the search query `{}`", query.text))?;

        Ok(())
    }
}
