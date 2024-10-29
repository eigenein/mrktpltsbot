use std::sync::atomic::{AtomicU64, Ordering};

use bon::Builder;
use maud::Render;

use crate::{
    db::{
        Db,
        search_query::{SearchQueries, SearchQuery},
    },
    marktplaats::{Marktplaats, SearchRequest},
    prelude::*,
    telegram::{
        Telegram,
        methods::{AllowedUpdate, GetMe, GetUpdates, Method, SendMessage},
        notification::SendNotification,
        objects::{Chat, ParseMode, ReplyParameters, Update, UpdatePayload},
        render::ListingCaption,
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
            let updates = GetUpdates::builder()
                .offset(self.offset.load(Ordering::Relaxed))
                .timeout_secs(self.poll_timeout_secs)
                .allowed_updates(&[AllowedUpdate::Message])
                .build()
                .call_on(&self.telegram)
                .await?;
            info!(n = updates.len(), "Received Telegram updates");

            for update in updates {
                self.offset.store(update.id + 1, Ordering::Relaxed);
                if let Err(error) = self.on_update(&me, &update).await {
                    error!("Failed to handle the update #{}: {error:#}", update.id);
                }
            }
        }
    }

    #[instrument(skip_all, err(level = Level::DEBUG))]
    async fn on_update(&self, me: &str, update: &Update) -> Result {
        let UpdatePayload::Message(message) = &update.payload else {
            bail!("The bot should only receive message updates")
        };
        let (Some(chat), Some(text)) = (&message.chat, &message.text) else {
            bail!("Message without an associated chat or text");
        };
        info!(update.id, chat.id, text, "Received");

        let reply_parameters = ReplyParameters::builder()
            .message_id(message.id)
            .allow_sending_without_reply(true)
            .build();
        self.on_message(me, chat, text, reply_parameters).await?; // TODO: inspect errors to user.

        info!(update.id, "Done");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn on_message(
        &self,
        me: &str,
        chat: &Chat,
        text: &str,
        reply_parameters: ReplyParameters,
    ) -> Result {
        if text.starts_with('/') {
            self.handle_command(text, chat.id, reply_parameters).await
        } else {
            self.on_search(me, text, chat.id, reply_parameters).await
        }
    }

    /// Handle the search request from Telegram.
    ///
    /// A search request is just a message that is not a command.
    #[instrument(skip_all)]
    async fn on_search(
        &self,
        me: &str,
        query: &str,
        chat_id: i64,
        reply_parameters: ReplyParameters,
    ) -> Result {
        let request = SearchRequest::standard(query, 1);
        let mut listings = self.marktplaats.search(&request).await?;
        info!(query, n_listings = listings.inner.len());

        let query = SearchQuery::from(query);
        SearchQueries(&mut *self.db.connection().await)
            .upsert(&query)
            .await?;

        // We need the subscribe command anyway, even if no listings were found.
        let subscribe_command = StartCommand::builder()
            .me(me)
            .text("Subscribe")
            .payload(StartPayload::subscribe_to(query.hash))
            .build();

        if let Some(listing) = listings.inner.pop() {
            let description = ListingCaption::builder()
                .listing(&listing)
                .search_query(query)
                .commands(&[subscribe_command])
                .build()
                .render()
                .into_string();
            SendNotification::builder()
                .chat_id(chat_id.into())
                .caption(&description)
                .pictures(&listing.pictures)
                .reply_parameters(reply_parameters)
                .build()
                .call_on(&self.telegram)
                .await?;
        } else {
            let _ = SendMessage::builder()
                .chat_id(chat_id)
                .text("There is no item matching the search query")
                .parse_mode(ParseMode::Html)
                .reply_parameters(reply_parameters)
                .build()
                .call_on(&self.telegram)
                .await?;
        }

        Ok(())
    }

    #[instrument(skip_all, name = "command")]
    async fn handle_command(
        &self,
        text: &str,
        chat_id: i64,
        reply_parameters: ReplyParameters,
    ) -> Result {
        Ok(())
    }
}
