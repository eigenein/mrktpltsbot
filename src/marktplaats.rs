use reqwest::Url;
use tracing::instrument;

use crate::{
    marktplaats::models::{Listing, SearchResponse},
    prelude::*,
};

mod models;

pub struct Marktplaats(reqwest::Client);

impl Marktplaats {
    pub fn new() -> Result<Self> {
        Ok(Self(
            reqwest::ClientBuilder::new()
                .user_agent("Googlebot/2.1 (+http://www.google.com/bot.html)")
                .build()
                .context("failed to build Marktplaats client")?,
        ))
    }

    /// Search for a single listing.
    ///
    /// If multiple listings match the query, only the first one is returned.
    #[instrument(skip_all, level = "debug", fields(query))]
    pub async fn find_one(&self, query: &str) -> Result<Option<Listing>> {
        let url = Url::parse_with_params(
            "https://www.marktplaats.nl/lrp/api/search?limit=1",
            &[("query", query)],
        )?;
        let response = self
            .0
            .get(url)
            .send()
            .await
            .with_context(|| format!("failed to search `{query}`"))?
            .error_for_status()
            .with_context(|| format!("failed to search `{query}` (bad status code)"))?
            .json::<SearchResponse>()
            .await
            .with_context(|| format!("failed to deserialize search response for `{query}`"))?
            .listings
            .pop();
        Ok(response)
    }
}
