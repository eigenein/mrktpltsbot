mod rate_limiter;

use chrono::Local;
use futures::{stream, StreamExt};
use tracing::instrument;

use crate::{
    marktplaats::{models::Listing, Marktplaats},
    prelude::*,
    tracing::to_option_traced,
};

pub struct Crawler {
    marktplaats: Marktplaats,
}

impl Crawler {
    pub fn new() -> Result<Self> {
        Ok(Self::with_marktplaats(Marktplaats::new()?))
    }

    pub const fn with_marktplaats(marktplaats: Marktplaats) -> Self {
        Self { marktplaats }
    }

    /// Run the crawler indefinitely.
    pub async fn run(&self) {
        stream::iter(2_068_900_000..)
            .filter_map(|item_id| async move { Some(self.crawl_item(item_id).await) })
            .filter_map(|result| async move { to_option_traced(result).flatten() })
            .for_each_concurrent(Some(1), |listing| async move {
                let lag = Local::now() - listing.timestamp;
                info!(%lag, "Search yielded an item");
            })
            .await;
    }

    /// Crawl a single item on Marktplaats.
    #[instrument(skip_all, fields(item_id = item_id))]
    async fn crawl_item(&self, item_id: u32) -> Result<Option<Listing>> {
        self.marktplaats
            .find_one(&format!("m{item_id}"))
            .await
            .with_context(|| format!("failed to fetch item #{item_id}"))
    }
}
