pub mod listing;

use bon::builder;
use reqwest::Url;
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
    #[instrument(skip_all, fields(query = request.query), ret(Display, level = Level::DEBUG), err(level = Level::DEBUG))]
    pub async fn search(&self, request: &SearchRequest<'_>) -> Result<String> {
        let query =
            serde_qs::to_string(request).context("failed to serialize the search request")?;
        let mut url = Url::parse("https://www.marktplaats.nl/lrp/api/search")?;
        url.set_query(Some(&query));
        self.0
            .get(url)
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
    pub offset: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

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
    #[allow(dead_code)]
    Optimized,

    #[serde(rename = "SORT_INDEX")]
    #[allow(dead_code)]
    SortIndex,

    #[serde(rename = "PRICE")]
    #[allow(dead_code)]
    Price,
}

#[must_use]
#[derive(Serialize)]
pub enum SortOrder {
    #[serde(rename = "INCREASING")]
    #[allow(dead_code)]
    Increasing,

    #[serde(rename = "DECREASING")]
    #[allow(dead_code)]
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
