use std::{borrow::Cow, time::Duration};

use bon::Builder;
use chrono::Utc;
use tokio::time::sleep;

use crate::{
    db,
    db::{Db, Item, Items, Notifications, SearchQuery, Subscription},
    marketplace::Marketplaces,
    prelude::*,
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

    marketplaces: Marketplaces,
}

impl SearchBot {
    /// Run the bot indefinitely.
    pub async fn run(self) {
        info!(
            "🔄 Running the search bot…",
            search_interval_secs = self.search_interval.as_secs_f64(),
        );
        let mut previous = None;
        loop {
            sleep(self.search_interval).await;
            match self
                .advance_and_handle(previous.as_ref())
                .await
                .context("failed to handle the next subscription")
            {
                Ok(handled) => {
                    previous = handled;
                }
                Err(error) => {
                    log::error!("‼️ Error: {error:#}");
                    capture_anyhow(&error);
                }
            }
        }
    }

    /// Advance in the subscription list and handle the subscription.
    ///
    /// # Returns
    ///
    /// Handled subscription entry as a next pointer.
    #[instrument(name = "⏩ Handling next subscription…", skip_all)]
    async fn advance_and_handle(
        &self,
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
            self.marketplaces.check_in().await;
            Ok(None)
        }
    }

    /// Handle the specified subscription.
    #[instrument(
        name = "🏭 Handling subscription…",
        skip_all,
        fields(
            subscription.chat_id = subscription.chat_id,
            search_query.hash = search_query.hash,
            search_query.text = search_query.text,
        ),
    )]
    async fn handle_subscription(
        &self,
        subscription: &Subscription,
        search_query: &SearchQuery,
    ) -> Result {
        let unsubscribe_link = self.command_builder.unsubscribe_link(search_query.hash);

        let items = self.marketplaces.search_infallible(search_query, None).await;
        info!("🛍️ Fetched items from all marketplaces", n_items = items.len());

        for item in items {
            let mut connection = self.db.connection().await;
            Items(&mut connection).upsert(Item { id: &item.id, updated_at: Utc::now() }).await?;
            let notification =
                db::Notification { item_id: item.id.clone(), chat_id: subscription.chat_id };
            if Notifications(&mut connection).exists(&notification).await? {
                debug!(
                    "✅ Notification was already sent",
                    chat_id = subscription.chat_id,
                    item_id = item.id,
                );
                continue;
            }
            info!("✉️ Notifying…", chat_id = subscription.chat_id, item_id = &notification.item_id);
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
            match telegram_notification
                .send_to(&self.telegram)
                .await
                .context("failed to send the notification")
            {
                Ok(()) => {
                    Notifications(&mut connection).upsert(&notification).await?;
                }
                Err(error) => {
                    log::error!("‼️ Error: {error:#}");
                    capture_anyhow(&error);
                }
            }
        }

        info!("✅ Done", chat_id = subscription.chat_id, text = &search_query.text);
        Ok(())
    }
}
