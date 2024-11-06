use std::time::Duration;

use bon::Builder;
use futures::{Stream, TryStreamExt};
use tokio_stream::StreamExt;

use crate::{
    db::Db,
    marktplaats::Marktplaats,
    prelude::*,
    telegram::{commands::CommandBuilder, reaction::Reaction},
};

/// Marktplaats reactor.
///
/// It crawls Marktplaats and produces reactions on the items.
#[derive(Builder)]
pub struct Reactor<'s> {
    db: &'s Db,
    marktplaats: &'s Marktplaats,
    command_builder: &'s CommandBuilder,
    crawl_interval: Duration,
}

impl<'s> Reactor<'s> {
    /// Run the reactor indefinitely and produce reactions.
    pub fn run(&'s self) -> impl Stream<Item = Result<Reaction<'static>>> + 's {
        info!(?self.crawl_interval, "Running Marktplaats reactor…");
        self.db
            .subscriptions()
            .throttle(self.crawl_interval)
            .try_filter_map(|entry| async move {
                match &entry {
                    Some((subscription, search_query)) => {
                        info!(subscription.chat_id, search_query.text, "Crawling…");
                    }
                    None => {
                        info!("No subscriptions to crawl");
                    }
                }
                Ok(None) // TODO
            })
    }
}
