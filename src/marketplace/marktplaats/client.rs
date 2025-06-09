use bon::Builder;
use reqwest_middleware::ClientWithMiddleware;
use serde::Serialize;
use url::Url;

use crate::{marketplace::marktplaats::Listings, prelude::*};

#[must_use]
#[derive(Clone)]
pub struct MarktplaatsClient(pub ClientWithMiddleware);

impl MarktplaatsClient {
    /// Search Marktplaats.
    pub async fn search(&self, request: &SearchRequest<'_>) -> Result<Listings> {
        info!(
            "🔎 Searching…",
            query = request.query.map(ToString::to_string),
            limit = request.limit.map(i64::from),
            in_title_and_description = request.search_in_title_and_description,
        );
        let url = {
            let query =
                serde_qs::to_string(request).context("failed to serialize the search request")?;
            let mut url = Url::parse("https://www.marktplaats.nl/lrp/api/search")?;
            url.set_query(Some(&query));
            url
        };
        self.0.get(url).send().await?.error_for_status()?.json().await.context("failed to search")
    }
}

#[must_use]
#[derive(Builder, Serialize)]
pub struct SearchRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    #[serde(rename = "sortBy", skip_serializing_if = "Option::is_none")]
    #[builder(required, default = Some(SortBy::SortIndex))]
    pub sort_by: Option<SortBy>,

    #[serde(rename = "sortOrder", skip_serializing_if = "Option::is_none")]
    #[builder(required, default = Some(SortOrder::Decreasing))]
    pub sort_order: Option<SortOrder>,

    #[serde(rename = "searchInTitleAndDescription", skip_serializing_if = "Option::is_none")]
    pub search_in_title_and_description: Option<bool>,

    #[serde(rename = "sellerIds")]
    #[builder(default)]
    pub seller_ids: &'a [u32],
}

impl SearchRequest<'_> {
    pub async fn call_on(&self, marktplaats: &MarktplaatsClient) -> Result<Listings> {
        marktplaats.search(self).await
    }
}

#[must_use]
#[derive(Serialize)]
pub enum SortBy {
    #[serde(rename = "OPTIMIZED")]
    #[expect(dead_code)]
    Optimized,

    #[serde(rename = "SORT_INDEX")]
    SortIndex,

    #[serde(rename = "PRICE")]
    #[expect(dead_code)]
    Price,
}

#[must_use]
#[derive(Serialize)]
pub enum SortOrder {
    #[serde(rename = "INCREASING")]
    #[expect(dead_code)]
    Increasing,

    #[serde(rename = "DECREASING")]
    Decreasing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_search_request_ok() -> Result {
        let request = SearchRequest::builder().build();
        assert_eq!(serde_qs::to_string(&request)?, "sortBy=SORT_INDEX&sortOrder=DECREASING");
        Ok(())
    }

    #[test]
    fn search_request_with_seller_ids_ok() -> Result {
        let request = SearchRequest::builder().seller_ids(&[42, 43]).build();
        assert_eq!(
            serde_qs::to_string(&request)?,
            "sortBy=SORT_INDEX&sortOrder=DECREASING&sellerIds[0]=42&sellerIds[1]=43",
        );
        Ok(())
    }
}
