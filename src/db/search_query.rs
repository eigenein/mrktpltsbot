use std::borrow::Cow;

use anyhow::Context;
use sqlx::{FromRow, SqliteConnection};

use crate::{marketplace::NormalisedQuery, prelude::*};

/// User's search query.
#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct SearchQuery {
    /// [SeaHash][1] of a search query.
    ///
    /// Used instead of the text where the payload size is limited (e.g. in `/start` payload).
    ///
    /// [1]: https://docs.rs/seahash/latest/seahash/
    pub hash: i64,

    pub text: String,
}

impl<S: AsRef<str>> From<S> for SearchQuery {
    #[expect(clippy::cast_possible_wrap)]
    fn from(text: S) -> Self {
        let text = text.as_ref();
        let normalised = NormalisedQuery::parse(text);
        let text = normalised.unparse();
        Self { hash: seahash::hash(text.as_bytes()) as i64, text }
    }
}

impl SearchQuery {
    pub fn normalised_query(&self) -> Cow<NormalisedQuery> {
        Cow::Owned(NormalisedQuery::parse(&self.text))
    }
}

pub struct SearchQueries<'a>(pub &'a mut SqliteConnection);

impl SearchQueries<'_> {
    #[instrument(
        name = "💾 Upserting search query…",
        level = Level::DEBUG,
        skip_all,
        fields(text = query.text, hash = query.hash),
    )]
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

    #[instrument(
        name = "💾 Fetching search query…",
        level = Level::DEBUG,
        skip_all,
        fields(hash = hash),
    )]
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

        let query = SearchQuery::from("test");
        search_queries.upsert(&query).await?;
        search_queries.upsert(&query).await?; // verify conflicts

        assert_eq!(search_queries.fetch_text(query.hash).await?, query.text);

        Ok(())
    }
}
