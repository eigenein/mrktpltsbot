use std::{
    fmt::{Display, Formatter},
    iter::once,
};

use async_trait::async_trait;
use bon::Builder;

use crate::{
    db::{Db, KeyValues, SearchQuery},
    heartbeat::Heartbeat,
    marketplace::{Marketplace, item::Item, vinted::search::SearchRequest},
    prelude::*,
};

mod client;
mod error;
mod search;

pub use self::{
    client::{AuthenticationTokens, VintedClient},
    error::Error as VintedError,
};

#[derive(Clone, Builder)]
pub struct Vinted {
    client: VintedClient,
    search_limit: u32,
    db: Db,
    heartbeat: Heartbeat,
}

impl Vinted {
    async fn refresh_tokens(&self, refresh_token: &str) -> Result<AuthenticationTokens> {
        let mut db = self.db.connection().await;
        let mut key_values = KeyValues(&mut db);
        match self.client.refresh_token(refresh_token).await {
            Ok(auth_tokens) => {
                key_values.upsert(&auth_tokens).await?;
                Ok(auth_tokens)
            }
            Err(error) => {
                bail!("failed to refresh the authentication token: {error:#}");
            }
        }
    }
}

impl Display for Vinted {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Vinted")
    }
}

#[async_trait]
impl Marketplace for Vinted {
    async fn check_in(&self) {
        self.heartbeat.check_in().await;
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<Item>> {
        let Some(auth_tokens) =
            KeyValues(&mut *self.db.connection().await).fetch::<AuthenticationTokens>().await?
        else {
            warn!("⚠️ Run `mrktpltsbot vinted authenticate` to use Vinted search");
            return Ok(vec![]);
        };
        let query = query.normalised_query();
        let search_text = query.search_text();
        let result = SearchRequest::builder()
            .search_text(&search_text)
            .per_page(self.search_limit)
            .access_token(&auth_tokens.access)
            .build()
            .call_on(&self.client)
            .await;
        let search_results = match result {
            Ok(search_results) => search_results,
            Err(VintedError::Reauthenticate) => {
                let auth_tokens = self.refresh_tokens(&auth_tokens.refresh).await?;
                SearchRequest::builder()
                    .search_text(&search_text)
                    .per_page(self.search_limit)
                    .access_token(&auth_tokens.access)
                    .build()
                    .call_on(&self.client)
                    .await?
            }
            Err(error) => {
                bail!("failed to search: {error:#}");
            }
        };
        let n_fetched = search_results.items.len();
        let items = search_results
            .items
            .into_iter()
            .filter(|item| {
                query.matches(item.title.split_whitespace().chain(once(item.brand_title.as_str())))
            })
            .map(Item::from)
            .collect::<Vec<Item>>();
        info!(
            "🛍️ Fetched from Vinted",
            search_text = search_text,
            n_fetched = n_fetched,
            n_filtered = items.len(),
        );
        Ok(items)
    }
}
