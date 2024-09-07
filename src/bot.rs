use std::sync::atomic::{AtomicU64, Ordering};

use backoff::ExponentialBackoff;
use bon::builder;

use crate::{
    db::Db,
    marktplaats::{Marktplaats, SearchRequest, SortBy, SortOrder},
    prelude::*,
    telegram::{
        listing::SendListingRequest,
        methods::{AllowedUpdate, GetUpdates, Method, SendMessage},
        objects::{ReplyParameters, Update, UpdatePayload},
        Telegram,
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
            self.handle_update(update).await?;
        }

        Ok(())
    }

    #[instrument(skip_all, fields(update.id = update.id))]
    async fn handle_update(&self, update: Update) -> Result {
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
            let request = SearchRequest::builder()
                .query(&text)
                .limit(1)
                .sort_by(SortBy::SortIndex)
                .sort_order(SortOrder::Decreasing)
                .search_in_title_and_description(true)
                .build();
            let mut listings = self.marktplaats.search(&request).await?;
            if let Some(listing) = listings.listings.pop() {
                SendListingRequest::with(chat.id.into(), &listing)
                    .call_on(&self.telegram)
                    .await?;
            } else {
                let _ = SendMessage::builder()
                    .chat_id(chat.id)
                    .text("There is no item matching the search query")
                    .reply_parameters(reply_parameters)
                    .build()
                    .call_on(&self.telegram)
                    .await?;
            }
        }

        Ok(())
    }
}
