mod client;
mod listing;

use async_trait::async_trait;
use bon::Builder;

use self::client::SearchRequest;
pub use self::{client::MarktplaatsClient, listing::Listings};
use crate::{
    db::SearchQuery,
    heartbeat::Heartbeat,
    marketplace::{Marketplace, item::Item},
    prelude::*,
};

#[must_use]
#[derive(Clone, Builder)]
pub struct Marktplaats {
    client: MarktplaatsClient,
    search_limit: u32,
    heartbeat: Heartbeat,
    search_in_title_and_description: bool,
}

#[async_trait]
impl Marketplace for Marktplaats {
    async fn check_in(&self) {
        self.heartbeat.check_in().await;
    }

    /// Search Marktplaats.
    async fn search(&self, query: &SearchQuery) -> Result<Vec<Item>> {
        let query = query.normalised_query();
        let search_text = query.search_text();
        let listings = SearchRequest::builder()
            .query(&search_text)
            .limit(self.search_limit)
            .search_in_title_and_description(self.search_in_title_and_description)
            .build()
            .call_on(&self.client)
            .await?
            .inner;
        let n_fetched = listings.len();
        let items = listings
            .into_iter()
            .filter(|listing| {
                query.matches(listing.title.split_whitespace().chain(listing.brand().into_iter()))
            })
            .map(TryInto::<Item>::try_into)
            .collect::<Result<Vec<Item>>>()?;
        info!(
            "🛍️ Fetched from Marktplaats",
            search_text = search_text,
            n_fetched = n_fetched,
            n_filtered = items.len(),
        );
        self.check_in().await;
        Ok(items)
    }
}
