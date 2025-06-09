//! Generic and shared stuff for different marketplace.

pub mod item;
mod marktplaats;
mod search;
mod search_bot;
mod vinted;

use std::fmt::Display;

use async_trait::async_trait;
use tokio::join;

pub use self::{
    marktplaats::{Marktplaats, MarktplaatsClient},
    search::NormalisedQuery,
    search_bot::SearchBot,
    vinted::{AuthenticationTokens as VintedAuthenticationTokens, Vinted, VintedClient},
};
use crate::{db::SearchQuery, marketplace::item::Item, prelude::*};

#[async_trait]
pub trait Marketplace: Display {
    async fn check_in(&self);

    async fn search(&self, query: &SearchQuery) -> Result<Vec<Item>>;

    #[instrument(
        name = "🔎 Searching on marketplace…",
        skip_all,
        fields(self = %self, query.text = query.text, limit = limit),
    )]
    async fn search_infallible(&self, query: &SearchQuery, limit: Option<usize>) -> Vec<Item> {
        match self.search(query).await.with_context(|| format!("failed to search on {self}")) {
            Ok(mut items) => {
                self.check_in().await;
                if let Some(limit) = limit {
                    items.truncate(limit);
                }
                items
            }
            Err(error) => {
                log::error!("‼️ Error: {error:#}");
                capture_anyhow(&error);
                Vec::new()
            }
        }
    }
}

#[derive(Clone)]
pub struct Marketplaces {
    pub marktplaats: Marktplaats,
    pub vinted: Vinted,
}

impl Marketplaces {
    pub async fn check_in(&self) {
        self.marktplaats.check_in().await;
        self.vinted.check_in().await;
    }

    pub async fn search_infallible(
        &self,
        query: &SearchQuery,
        marketplace_limit: Option<usize>,
    ) -> Vec<Item> {
        let (mut items_1, items_2) = join!(
            self.marktplaats.search_infallible(query, marketplace_limit),
            self.vinted.search_infallible(query, marketplace_limit)
        );
        items_1.extend(items_2);
        items_1
    }
}
