use std::sync::atomic::{AtomicU64, Ordering};

use backoff::ExponentialBackoff;
use bon::Builder;

use crate::{
    db::{Db, Insert, search_query::SearchQuery},
    marktplaats::{Marktplaats, SearchRequest, SortBy, SortOrder},
    prelude::*,
    telegram::{
        Telegram,
        methods::{AllowedUpdate, GetMe, GetUpdates, Method, SendMessage},
        notification::Notification,
        objects::{Chat, ReplyParameters, Update, UpdatePayload},
        render::{ListingCaption, TryRender},
        start::{StartCommand, StartPayload},
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
                    warn!("Failed to handle Telegram updates: {error:#}");
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

    #[instrument(skip_all, fields(update.id = update.id), err(level = Level::DEBUG))]
    async fn on_update(&self, me: &str, update: &Update) -> Result {
        let UpdatePayload::Message(message) = &update.payload else {
            bail!("The bot should only receive message updates")
        };
        let (Some(chat), Some(text)) = (&message.chat, &message.text) else {
            bail!("Message without an associated chat or text");
        };
        info!(?message.chat, message.text, "Received");

        let reply_parameters = ReplyParameters::builder()
            .message_id(message.id)
            .allow_sending_without_reply(true)
            .build();
        self.handle_message(me, chat, text, reply_parameters).await // TODO: report errors to user.
    }

    async fn handle_message(
        &self,
        me: &str,
        chat: &Chat,
        text: &str,
        reply_parameters: ReplyParameters,
    ) -> Result {
        if text.starts_with('/') {
            self.handle_command(text, chat.id, reply_parameters).await
        } else {
            self.handle_search(me, text, chat.id, reply_parameters)
                .await
        }
    }

    /// Handle the search request from Telegram.
    ///
    /// A search request is just a message that is not a command.
    async fn handle_search(
        &self,
        me: &str,
        query: &str,
        chat_id: i64,
        reply_parameters: ReplyParameters,
    ) -> Result {
        let query = SearchQuery::from(query);
        self.db.insert(&query).await?;
        let request = SearchRequest::builder()
            .query(&query.text)
            .limit(1)
            .sort_by(SortBy::SortIndex)
            .sort_order(SortOrder::Decreasing)
            .search_in_title_and_description(true)
            .build();
        let mut listings = self.marktplaats.search(&request).await?;
        if let Some(listing) = listings.inner.pop() {
            let subscribe_command = StartCommand::builder()
                .me(me)
                .text("Subscribe")
                .payload(StartPayload::subscribe_to(query.hash))
                .build();
            let caption = ListingCaption::builder()
                .listing(&listing)
                .search_query(query)
                .commands(&[subscribe_command])
                .build()
                .try_render()?
                .into_string();
            Notification::builder()
                .chat_id(chat_id.into())
                .caption(&caption)
                .pictures(&listing.pictures)
                .reply_parameters(reply_parameters)
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

    async fn handle_command(
        &self,
        text: &str,
        chat_id: i64,
        reply_parameters: ReplyParameters,
    ) -> Result {
        Ok(())
    }
}
