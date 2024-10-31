use std::{
    collections::HashSet,
    sync::atomic::{AtomicU64, Ordering},
};

use bon::bon;
use maud::Render;

use crate::{
    db::{
        Db,
        query_hash::QueryHash,
        search_query::{SearchQueries, SearchQuery},
        subscription::{Subscription, Subscriptions},
    },
    marktplaats::{Marktplaats, SearchRequest},
    prelude::*,
    telegram::{
        Telegram,
        commands::{CommandBuilder, CommandPayload, SubscriptionStartCommand},
        methods::{AllowedUpdate, GetMe, GetUpdates, Method, SendMessage, SetMyDescription},
        notification::SendNotification,
        objects::{ChatId, LinkPreviewOptions, ParseMode, ReplyParameters, Update, UpdatePayload},
        render,
        render::{simple_message, unauthorized},
    },
};

pub struct Bot {
    telegram: Telegram,
    authorized_chat_ids: HashSet<i64>,
    db: Db,
    marktplaats: Marktplaats,
    poll_timeout_secs: u64,
    offset: AtomicU64,
    command_builder: CommandBuilder,
}

#[bon]
impl Bot {
    #[builder(finish_fn = try_connect)]
    pub async fn new(
        telegram: Telegram,
        db: Db,
        authorized_chat_ids: HashSet<i64>,
        marktplaats: Marktplaats,
        poll_timeout_secs: u64,
        offset: u64,
    ) -> Result<Self> {
        let me = GetMe
            .call_on(&telegram)
            .await
            .context("failed to get bot’s user")?
            .username
            .context("the bot has no username")?;
        info!(me, "Successfully connected to Telegram Bot API");

        SetMyDescription::builder()
            .description("👋 This is a private bot for Marktplaats\n\nFeel free to set up your own instance from https://github.com/eigenein/mrktpltsbot")
            .build()
            .call_on(&telegram)
            .await
            .context("failed to set the bot description")?;

        let this = Self {
            telegram,
            db,
            marktplaats,
            poll_timeout_secs,
            authorized_chat_ids,
            offset: AtomicU64::new(offset),
            command_builder: CommandBuilder::new(&me)?,
        };
        Ok(this)
    }
}

impl Bot {
    pub async fn run_telegram(&self) -> Result {
        info!("Running Telegram bot…");
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
                let update_id = update.id;
                if let Err(error) = self.on_update(update).await {
                    error!("Failed to handle the update #{update_id}: {error:#}");
                }
            }
        }
    }

    #[instrument(skip_all, err(level = Level::DEBUG))]
    async fn on_update(&self, update: Update) -> Result {
        let UpdatePayload::Message(message) = update.payload else {
            bail!("The bot should only receive message updates")
        };
        let (Some(chat), Some(text)) = (message.chat, message.text) else {
            bail!("Message without an associated chat or text");
        };
        let text = text.trim();
        info!(update.id, ?chat.id, text, "Received");

        let reply_parameters = ReplyParameters::builder()
            .message_id(message.id)
            .allow_sending_without_reply(true)
            .build();
        self.on_message(chat.id, text, reply_parameters).await?; // TODO: inspect errors to user.

        info!(update.id, "Done");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn on_message(
        &self,
        chat_id: ChatId,
        text: &str,
        reply_parameters: ReplyParameters,
    ) -> Result {
        let chat_id = match chat_id {
            ChatId::Integer(chat_id) if self.authorized_chat_ids.contains(&chat_id) => chat_id,
            _ => {
                // TODO: support username chat IDs.
                warn!(?chat_id, "Unauthorized");
                let _ = SendMessage::builder()
                    .chat_id(&chat_id)
                    .text(unauthorized(&chat_id).render().into_string())
                    .parse_mode(ParseMode::Html)
                    .link_preview_options(LinkPreviewOptions::DISABLED)
                    .build()
                    .call_on(&self.telegram)
                    .await?;
                return Ok(());
            }
        };
        if text.starts_with('/') {
            self.handle_command(text, chat_id, reply_parameters).await
        } else {
            self.on_search(text.to_lowercase(), chat_id, reply_parameters)
                .await
        }
    }

    /// Handle the search request from Telegram.
    ///
    /// A search request is just a message that is not a command.
    #[instrument(skip_all)]
    async fn on_search(
        &self,
        query: String,
        chat_id: i64,
        reply_parameters: ReplyParameters,
    ) -> Result {
        let query = SearchQuery::from(query);
        let request = SearchRequest::standard(&query.text, 1);
        let mut listings = self.marktplaats.search(&request).await?;
        info!(
            text = query.text,
            hash = query.hash.0,
            n_listings = listings.inner.len()
        );

        SearchQueries(&mut *self.db.connection().await)
            .upsert(&query)
            .await?;

        // We need the subscribe command anyway, even if no listings were found.
        let command_payload = CommandPayload::builder()
            .subscribe(SubscriptionStartCommand::new(query.hash))
            .build();
        let subscribe_link = self
            .command_builder
            .link()
            .payload(&command_payload)
            .content("Subscribe")
            .build();

        if let Some(listing) = listings.inner.pop() {
            let description = render::listing_description()
                .listing(&listing)
                .search_query(&query)
                .links(&[subscribe_link])
                .render();
            SendNotification::builder()
                .chat_id(&chat_id.into())
                .caption(&description)
                .pictures(&listing.pictures)
                .reply_parameters(reply_parameters)
                .build()
                .call_on(&self.telegram)
                .await?;
        } else {
            let text = render::simple_message()
                .markup("There are no items matching the search query. Try a different query or subscribe anyway to wait for them to appear")
                .links(&[subscribe_link])
                .render();
            let _ = SendMessage::builder()
                .chat_id(&chat_id.into())
                .text(text)
                .parse_mode(ParseMode::Html)
                .reply_parameters(reply_parameters)
                .link_preview_options(LinkPreviewOptions::DISABLED)
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
        if text == "/start" {
            // Just an initial greeting.
            let _ = SendMessage::builder()
                .chat_id(&chat_id.into())
                .text("👋")
                .build()
                .call_on(&self.telegram)
                .await?;
            let _ = SendMessage::builder()
                .chat_id(&chat_id.into())
                .text("Just send me a search query to start")
                .build()
                .call_on(&self.telegram)
                .await?;
        } else if let Some(payload) = text.strip_prefix("/start ") {
            // Command with a payload.
            let command = CommandPayload::from_base64(payload)?;
            debug!(?command, "Received command");

            if let Some(subscribe) = command.subscribe {
                // Subscribe to the search query.
                info!(chat_id, subscribe.query_hash, "Unsubscribe");
                let query_hash = QueryHash(subscribe.query_hash);
                let subscription = Subscription {
                    query_hash,
                    chat_id,
                };
                Subscriptions(&mut *self.db.connection().await)
                    .upsert(&subscription)
                    .await?;
                let unsubscribe_link = self
                    .command_builder
                    .link()
                    .content("Unsubscribe")
                    .payload(
                        &CommandPayload::builder()
                            .unsubscribe(SubscriptionStartCommand::new(query_hash))
                            .build(),
                    )
                    .build();
                let text = simple_message()
                    .markup("✅ You are now subscribed")
                    .links(&[unsubscribe_link])
                    .render();
                let _ = SendMessage::builder()
                    .chat_id(&chat_id.into())
                    .text(text)
                    .parse_mode(ParseMode::Html)
                    .link_preview_options(LinkPreviewOptions::DISABLED)
                    .build()
                    .call_on(&self.telegram)
                    .await?;
            }

            if let Some(unsubscribe) = command.unsubscribe {
                // Unsubscribe from the search query.
                // TODO
            }
        } else {
            // Unknown command.
            let _ = SendMessage::builder()
                .chat_id(&chat_id.into())
                .text("I am sorry, but I do not know this command")
                .reply_parameters(reply_parameters)
                .build()
                .call_on(&self.telegram)
                .await?;
        }
        Ok(())
    }
}
