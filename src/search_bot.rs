use crate::{
    marktplaats,
    marktplaats::{SearchListing, SearchResponse},
    prelude::*,
    redis::{Notification, check_seen, pick_random_subscription},
    telegram::{format::format_listing_text, types::InlineKeyboardButton},
};

const SEARCH_LIMIT: &str = "30";

#[must_use]
pub struct Bot {
    redis: RedisConnection,
    polling_interval: Duration,
}

impl Bot {
    pub fn new(redis: RedisConnection, polling_interval_secs: u64) -> Self {
        Self {
            redis,
            polling_interval: Duration::from_secs(polling_interval_secs),
        }
    }

    pub async fn run(mut self) -> Result {
        info!("Running…");
        loop {
            if let Some((subscription_id, chat_id, query)) =
                pick_random_subscription(&mut self.redis).await?
            {
                self.check_subscription(subscription_id, chat_id, query).await.log_result();
            }
            task::sleep(self.polling_interval).await;
        }
    }

    /// Perform the related search and handle search results.
    async fn check_subscription(
        &mut self,
        subscription_id: i64,
        chat_id: i64,
        query: String,
    ) -> Result {
        info!("Checking `{}`…", query);
        self.handle_search_result(
            subscription_id,
            chat_id,
            marktplaats::search(&query, SEARCH_LIMIT).await?,
        )
        .await?;
        Ok(())
    }

    async fn handle_search_result(
        &mut self,
        subscription_id: i64,
        chat_id: i64,
        response: SearchResponse,
    ) -> Result {
        info!("{} search results.", response.listings.len());

        for listing in response.listings {
            if check_seen(&mut self.redis, chat_id, &listing.item_id).await? {
                info!("New item: {} | {}", listing.item_id, listing.title);
                push_notification(&mut self.redis, Some(subscription_id), chat_id, &listing)
                    .await?;
            }
        }

        Ok(())
    }
}

/// Push the notification to the queue.
///
/// # Arguments
///
/// - `subscription_id`: Optional subscription ID, adds the unsubscribe button if present.
/// - `chat_id`: Target chat ID.
/// - `listing`: Marktplaats search listing.
pub async fn push_notification(
    redis: &mut RedisConnection,
    subscription_id: Option<i64>,
    chat_id: i64,
    listing: &SearchListing,
) -> Result {
    let mut buttons = vec![InlineKeyboardButton::new_url_button(format!(
        "https://www.marktplaats.nl{}",
        listing.url
    ))];
    if let Some(subscription_id) = subscription_id {
        buttons.push(InlineKeyboardButton::new_unsubscribe_button(subscription_id, None));
    }
    crate::redis::push_notification(
        redis,
        Notification {
            chat_id,
            text: format_listing_text(listing),
            image_urls: listing.image_urls().map(ToString::to_string).collect(),
            reply_markup: Some(buttons.into()),
        },
    )
    .await?;
    Ok(())
}
