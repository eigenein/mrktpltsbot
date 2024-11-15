use std::{borrow::Cow, future, time::Duration};

use bon::Builder;
use chrono::Utc;
use futures::{Stream, StreamExt, TryStreamExt, stream};

use crate::{
    db::{
        Db,
        item::{Item, Items},
        notification::{Notification, Notifications},
        search_query::SearchQuery,
        subscription::Subscription,
    },
    marktplaats::{Marktplaats, SearchRequest},
    prelude::*,
    telegram::{
        commands::CommandBuilder,
        objects::ParseMode,
        reaction::{Reaction, ReactionMethod},
        render,
        render::ManageSearchQuery,
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
        tokio_stream::StreamExt::throttle(self.db.subscriptions(), self.crawl_interval)
            .try_filter_map(|entry| {
                if entry.is_none() {
                    info!("No subscriptions found");
                }
                future::ready(Ok(entry))
            })
            .and_then(move |(subscription, search_query)| async move {
                Ok(stream::iter(self.on_subscription(subscription, search_query).await?).map(Ok))
            })
            .try_flatten()
    }

    /// Handle the [`Subscription`] and return [`Reaction`]'s to it.
    #[instrument(skip_all)]
    async fn on_subscription(
        &self,
        subscription: Subscription,
        search_query: SearchQuery,
    ) -> Result<Vec<Reaction<'static>>> {
        info!(subscription.chat_id, search_query.text, "Crawling…");
        let text = &search_query.text;
        let unsubscribe_link = &self.command_builder.unsubscribe_link(search_query.hash);

        let listings = SearchRequest::standard(&search_query.text, self.search_limit)
            .call_on(self.marktplaats)
            .await?
            .inner;
        info!(subscription.chat_id, search_query.text, n_listings = listings.len(), "Fetched");

        let reactions: Vec<_> = stream::iter(listings)
            .map(Ok)
            .try_filter_map(move |listing| async move {
                let mut connection = self.db.connection().await;
                let item = Item { id: &listing.item_id, updated_at: Utc::now() };
                Items(&mut connection).upsert(item).await?;
                let notification = Notification {
                    item_id: listing.item_id.clone(),
                    chat_id: subscription.chat_id,
                };
                if Notifications(&mut connection).exists(&notification).await? {
                    trace!(subscription.chat_id, listing.item_id, "Notification was already sent");
                    return Ok(None);
                }
                info!(subscription.chat_id, notification.item_id, "Reacting");
                let description = render::listing_description(
                    &listing,
                    &ManageSearchQuery::new(text, &[unsubscribe_link]),
                );
                let reaction_method = ReactionMethod::from_listing()
                    .chat_id(Cow::Owned(subscription.chat_id.into()))
                    .text(description)
                    .pictures(listing.pictures)
                    .parse_mode(ParseMode::Html)
                    .build();
                Ok(Some(Reaction {
                    methods: vec![reaction_method],
                    notification: Some(notification),
                }))
            })
            .filter_map(|result| async {
                // Log and skip errors so that we would still process the rest of the listings.
                result.inspect_err(|error: &Error| error!("Failed to react: {error:#}")).ok()
            })
            .collect()
            .await;

        info!(subscription.chat_id, search_query.text, n_reactions = reactions.len(), "Done");
        Ok(reactions)
    }
}
