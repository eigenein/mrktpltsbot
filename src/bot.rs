pub mod query;

use std::sync::atomic::{AtomicU64, Ordering};

use backoff::ExponentialBackoff;
use bon::Builder;

use crate::{
    bot::query::SearchQuery,
    db::Db,
    marktplaats::{Marktplaats, SearchRequest, SortBy, SortOrder},
    prelude::*,
    telegram::{
        Telegram,
        methods::{AllowedUpdate, GetMe, GetUpdates, Method, SendMessage},
        notification::Notification,
        objects::{ReplyParameters, Update, UpdatePayload},
    },
};

#[derive(Builder)]
pub struct Bot {
    telegram: Telegram,
    db: Db,
    marktplaats: Marktplaats,
    poll_timeout_secs: u64,

    #[builder(default = AtomicU64::new(0))]
    offset: AtomicU64,
}

impl Bot {
    pub async fn run_telegram(&self) -> Result {
        let me = self
            .telegram
            .call(&GetMe)
            .await
            .context("failed to get bot’s user")?
            .username
            .context("the bot has no username")?;

        info!(me, "Running Telegram bot…");
        loop {
            backoff::future::retry_notify(
                ExponentialBackoff::default(),
                || async { Ok(self.handle_telegram_updates(&me).await?) },
                |error, _| {
                    warn!("Bot iteration failed: {error:#}");
                },
            )
            .await
            .context("fatal error")?;
        }
    }

    #[instrument(skip_all)]
    async fn handle_telegram_updates(&self, me: &str) -> Result {
        let updates = GetUpdates::builder()
            .offset(self.offset.load(Ordering::Relaxed))
            .timeout_secs(self.poll_timeout_secs)
            .allowed_updates(&[AllowedUpdate::Message])
            .build()
            .call_on(&self.telegram)
            .await?;
        info!(n_updates = updates.len(), "Received");

        for update in updates {
            self.offset.store(update.id + 1, Ordering::Relaxed);
            self.on_update(me, &update)
                .await
                .with_context(|| format!("failed to handle the update #{}", update.id))?;
        }

        Ok(())
    }

    #[instrument(skip_all, fields(update.id = update.id))]
    async fn on_update(&self, me: &str, update: &Update) -> Result {
        let UpdatePayload::Message(message) = &update.payload else {
            panic!("The bot should only receive message updates")
        };
        info!(?message.chat, message.text, "Received");

        let (Some(chat), Some(text)) = (&message.chat, &message.text) else {
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
            self.handle_quick_search(me, &text.trim().to_lowercase(), chat.id, reply_parameters)
                .await?;
        }

        Ok(())
    }

    /// Handle the quick search request from Telegram.
    async fn handle_quick_search(
        &self,
        me: &str,
        query: &str,
        chat_id: i64,
        reply_parameters: ReplyParameters,
    ) -> Result {
        let query = SearchQuery::from(query);
        self.db.insert_search_query(query).await?;
        let request = SearchRequest::builder()
            .query(query.text)
            .limit(1)
            .sort_by(SortBy::SortIndex)
            .sort_order(SortOrder::Decreasing)
            .search_in_title_and_description(true)
            .build();
        let mut listings = self.marktplaats.search(&request).await?;
        if let Some(listing) = listings.inner.pop() {
            Notification::builder()
                .chat_id(chat_id.into())
                .listing(&listing)
                .reply_parameters(reply_parameters)
                .query(query)
                .me(me)
                .build()
                .send_with(&self.telegram)
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
