use std::sync::atomic::{AtomicU64, Ordering};

use backoff::ExponentialBackoff;
use bon::builder;

use crate::{
    db::Db,
    marktplaats::{Marktplaats, SearchRequest, SortBy, SortOrder},
    prelude::*,
    telegram::{
        Telegram,
        listing::ListingView,
        methods::{AllowedUpdate, GetUpdates, Method, SendMessage},
        objects::{ReplyParameters, Update, UpdatePayload},
    },
};

#[builder]
pub struct Bot {
    telegram: Telegram,
    db: Db,
    marktplaats: Marktplaats,
    timeout_secs: u64,

    #[builder(default = AtomicU64::new(0))]
    offset: AtomicU64,
}

impl Bot {
    pub async fn run_telegram(&self) -> Result {
        info!("Running Telegram bot…");
        loop {
            backoff::future::retry_notify(
                ExponentialBackoff::default(),
                || async { Ok(self.handle_telegram_updates().await?) },
                |error, _| {
                    warn!("Bot iteration failed: {error:#}");
                },
            )
            .await
            .context("fatal error")?;
        }
    }

    #[instrument(skip_all)]
    async fn handle_telegram_updates(&self) -> Result {
        let updates = GetUpdates::builder()
            .offset(self.offset.load(Ordering::Relaxed))
            .timeout_secs(self.timeout_secs)
            .allowed_updates(&[AllowedUpdate::Message])
            .build()
            .call_on(&self.telegram)
            .await?;

        for update in updates {
            info!(update.id, "Received update");
            self.offset.store(update.id + 1, Ordering::Relaxed);
            self.on_update(update).await?;
        }

        Ok(())
    }

    #[instrument(skip_all, fields(update.id = update.id))]
    async fn on_update(&self, update: Update) -> Result {
        let UpdatePayload::Message(message) = update.payload else {
            unreachable!("message is the only allowed update type")
        };
        info!(?message.chat, message.text, "Received");

        let (Some(chat), Some(text)) = (message.chat, message.text) else {
            warn!("Message without an associated chat or text");
            return Ok(());
        };

        let reply_parameters = ReplyParameters::builder()
            .message_id(message.id)
            .allow_sending_without_reply(true)
            .build();

        if text.starts_with('/') {
            let _ = SendMessage::builder()
                .chat_id(chat.id)
                .text("I can't answer commands just yet")
                .reply_parameters(reply_parameters)
                .build()
                .call_on(&self.telegram)
                .await?;
        } else {
            self.handle_search(&text.trim().to_lowercase(), chat.id, reply_parameters)
                .await?;
        }

        Ok(())
    }

    async fn handle_search(
        &self,
        query: &str,
        chat_id: i64,
        reply_parameters: ReplyParameters,
    ) -> Result {
        self.db.insert_search_query(query).await?;
        let request = SearchRequest::builder()
            .query(query)
            .limit(1)
            .sort_by(SortBy::SortIndex)
            .sort_order(SortOrder::Decreasing)
            .search_in_title_and_description(true)
            .build();
        let mut listings = self.marktplaats.search(&request).await?;
        if let Some(listing) = listings.inner.pop() {
            ListingView::builder()
                .chat_id(chat_id)
                .listing(&listing)
                .reply_parameters(reply_parameters)
                .build()
                .call_on(&self.telegram)
                .await?;
        } else {
            let _ = SendMessage::builder()
                .chat_id(chat_id)
                .text("There is no item matching the search query")
                .reply_parameters(reply_parameters)
                .build()
                .call_on(&self.telegram)
                .await?;
        }
        Ok(())
    }
}
