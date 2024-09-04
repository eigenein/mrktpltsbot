pub mod listing;

use bon::builder;
use serde::Serialize;

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

#[must_use]
#[builder]
#[derive(Serialize)]
pub struct SearchRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,

    #[serde(rename = "sortBy", skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SortBy>,

    #[serde(rename = "sortOrder", skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,

    #[serde(
        rename = "searchInTitleAndDescription",
        skip_serializing_if = "Option::is_none"
    )]
    pub search_in_title_and_description: Option<bool>,

    #[serde(rename = "sellerIds")]
    #[builder(default)]
    pub seller_ids: &'a [u32],
}

#[must_use]
#[derive(Serialize)]
pub enum SortBy {
    #[serde(rename = "OPTIMIZED")]
    Optimized,

    #[serde(rename = "SORT_INDEX")]
    SortIndex,

    #[serde(rename = "PRICE")]
    Price,
}

#[must_use]
#[derive(Serialize)]
pub enum SortOrder {
    #[serde(rename = "INCREASING")]
    Increasing,

    #[serde(rename = "DECREASING")]
    Decreasing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seller_ids_ok() -> Result {
        let request = SearchRequest::builder().seller_ids(&[42, 43]).build();
        assert_eq!(
            serde_qs::to_string(&request)?,
            "sellerIds[0]=42&sellerIds[1]=43"
        );
        Ok(())
    }

    #[test]
    fn search_in_title_and_description_ok() -> Result {
        let request = SearchRequest::builder()
            .search_in_title_and_description(true)
            .build();
        assert_eq!(
            serde_qs::to_string(&request)?,
            "searchInTitleAndDescription=true"
        );
        Ok(())
    }
}
