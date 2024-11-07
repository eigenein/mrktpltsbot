use std::{borrow::Cow, future, time::Duration};

use bon::Builder;
use futures::{Stream, TryStreamExt, stream};
use tokio_stream::StreamExt;

use crate::{
    db::{
        Db,
        notification::{Notification, Notifications},
    },
    marktplaats::{Marktplaats, SearchRequest},
    prelude::*,
    telegram::{
        commands::CommandBuilder,
        objects::ParseMode,
        reaction::{Reaction, ReactionMethod},
        render,
    },
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
    search_limit: u32,
}

impl<'s> Reactor<'s> {
    /// Run the reactor indefinitely and produce reactions.
    pub fn run(&'s self) -> impl Stream<Item = Result<Reaction<'static>>> + 's {
        info!(?self.crawl_interval, "Running Marktplaats reactor…");
        self.db
            .subscriptions()
            .throttle(self.crawl_interval)
            .try_filter_map(|entry| {
                match &entry {
                    Some((subscription, search_query)) => {
                        info!(subscription.chat_id, search_query.text, "Crawling…");
                    }
                    None => {
                        info!("No subscriptions to crawl");
                    }
                }
                future::ready(Ok(entry))
            })
            .try_filter_map(|(subscription, search_query)| async {
                // let description = render::listing_description()
                //     .listing(&listing)
                //     .search_query(&search_query.text)
                //     .links(&[self.command_builder.unsubscribe_link(search_query.hash)])
                //     .render();
                // ReactionMethod::from_listing()
                //     .chat_id(Cow::Owned(chat_id.into()))
                //     .text(description)
                //     .pictures(listing.pictures)
                //     .reply_parameters(reply_parameters)
                //     .parse_mode(ParseMode::Html)
                //     .build()
                //     .into()
                Ok(None) // TODO
            })
    }
}
