use std::time::Duration;

use reqwest::Url;
use tracing::instrument;

use crate::{
    marktplaats::models::{Listing, SearchResponse},
    prelude::*,
};

pub mod models;

pub struct Marktplaats(reqwest::Client);

impl Marktplaats {
    const TIMEOUT: Duration = Duration::from_secs(10);

    pub fn new() -> Result<Self> {
        Ok(Self(
            reqwest::ClientBuilder::new()
                .user_agent("Googlebot/2.1 (+http://www.google.com/bot.html)")
                .connect_timeout(Self::TIMEOUT)
                .timeout(Self::TIMEOUT)
                .build()
                .context("failed to build Marktplaats client")?,
        ))
    }

    /// Search for a single listing.
    ///
    /// If multiple listings match the query, only the first one is returned.
    #[instrument(skip_all, fields(query = query))]
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
