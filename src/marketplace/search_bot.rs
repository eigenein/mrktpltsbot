use std::{borrow::Cow, time::Duration};

use bon::Builder;
use chrono::Utc;
use tokio::time::sleep;
use tracing::{error, info};

use crate::{
    db,
    db::{Db, Item, Items, Notifications, SearchQuery, Subscription},
    marketplace::{Marketplace, marktplaats::Marktplaats, vinted::Vinted},
    prelude::{instrument, *},
    telegram::{
        Telegram,
        TelegramNotification,
        commands::CommandBuilder,
        objects::ParseMode,
        render,
        render::ManageSearchQuery,
    },
};

/// Core logic of the search bot.
#[derive(Builder)]
pub struct SearchBot {
    db: Db,

    command_builder: CommandBuilder, // TODO: should it belong in `Telegram`?

    /// Search interval between subscriptions.
    search_interval: Duration,

    /// Telegram connection.
    telegram: Telegram,

    /// Marktplaats connection.
    marktplaats: Marktplaats,

    /// Vinted connection.
    vinted: Vinted,
}

impl SearchBot {
    /// Run the bot indefinitely.
    pub async fn run(mut self) {
        info!(?self.search_interval, "🔄 Running the search bot…");
        let mut previous = None;
        loop {
            sleep(self.search_interval).await;
            match self.advance_and_handle(previous.as_ref()).await {
                Ok(handled) => {
                    previous = handled;
                }
                Err(error) => {
                    error!("‼️ Failed to handle the next subscription: {error:#}");
                }
            }
        }
    }

    /// Advance in the subscription list and handle the subscription.
    ///
    /// # Returns
    ///
    /// Handled subscription entry as a next pointer.
    async fn advance_and_handle(
        &mut self,
        previous: Option<&(Subscription, SearchQuery)>,
    ) -> Result<Option<(Subscription, SearchQuery)>> {
        let current = match previous {
            Some((previous, _)) => match self.db.next_subscription(previous).await? {
                Some(next) => Some(next),
                None => self.db.first_subscription().await?, // reached the end, restart
            },
            None => self.db.first_subscription().await?, // fresh start or no subscriptions
        };
        if let Some((subscription, search_query)) = &current {
            self.handle_subscription(subscription, search_query).await?;
            Ok(current)
        } else {
            info!("📭 No active subscriptions");
            self.marktplaats.check_in().await;
            self.vinted.check_in().await;
            Ok(None)
        }
    }

    /// Handle the specified subscription.
    #[instrument(skip_all)]
    async fn handle_subscription(
        &mut self,
        subscription: &Subscription,
        search_query: &SearchQuery,
    ) -> Result {
        info!(subscription.chat_id, search_query.text, "🏭 Handling…");
        let unsubscribe_link = self.command_builder.unsubscribe_link(search_query.hash);

        let mut items = Vec::new();
        self.marktplaats.search_and_extend_infallible(search_query, None, &mut items).await;
        self.vinted.search_and_extend_infallible(search_query, None, &mut items).await;

        info!(n_items = items.len(), "🛍️ Fetched from all marketplaces");
        for item in items {
            let mut connection = self.db.connection().await;
            Items(&mut connection).upsert(Item { id: &item.id, updated_at: Utc::now() }).await?;
            let notification =
                db::Notification { item_id: item.id.clone(), chat_id: subscription.chat_id };
            if Notifications(&mut connection).exists(&notification).await? {
                trace!(subscription.chat_id, item.id, "✅ Notification was already sent");
                continue;
            }
            info!(subscription.chat_id, notification.item_id, "✉️ Notifying…");
            let description = render::item_description(
                &item,
                &ManageSearchQuery::new(&search_query.text, &[&unsubscribe_link]),
            );
            let telegram_notification = TelegramNotification::builder()
                .chat_id(Cow::Owned(subscription.chat_id.into()))
                .text(description.into())
                .maybe_picture_url(item.picture_url.as_ref())
                .parse_mode(ParseMode::Html)
                .build();
            match telegram_notification.send_to(&self.telegram).await {
                Ok(()) => {
                    Notifications(&mut connection).upsert(&notification).await?;
                }
                Err(error) => {
                    error!("failed to send the notification: {error:#}");
                }
            }
        }

        info!(subscription.chat_id, search_query.text, "✅ Done");
        Ok(())
    }
}
