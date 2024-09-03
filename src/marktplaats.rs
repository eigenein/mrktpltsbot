pub mod listing;

use crate::prelude::*;

#[must_use]
pub struct Marktplaats(pub reqwest::Client);

impl Marktplaats {
    /// Search Marktplaats.
    ///
    /// # Returns
    ///
    /// Raw response payload.
    #[instrument(skip_all, fields(query = query, limit = limit), ret(level = Level::DEBUG), err(level = Level::DEBUG))]
    pub async fn search(&self, query: &str, limit: u32) -> Result<String> {
        self.0
            .get("https://www.marktplaats.nl/lrp/api/search?offset=0&sortBy=SORT_INDEX&sortOrder=DECREASING")
            .query(&[("query", query)])
            .query(&[("limit", limit)])
            .send()
            .await
            .with_context(|| format!("failed to search `{query}`"))?
            .error_for_status()?
            .text()
            .await
            .with_context(|| format!("failed to read search response for `{query}`"))
    }
}
