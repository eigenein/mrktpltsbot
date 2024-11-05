pub mod telegram;

use std::collections::HashSet;

use bon::bon;
use futures::TryStreamExt;
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
        methods::{GetMe, Method, SendMessage, SendNotification, SetMyDescription},
        objects::{ChatId, LinkPreviewOptions, Message, ParseMode, ReplyParameters},
        render,
    },
};

#[deprecated = "It is being split up"]
pub struct Bot {
    db: Db,
    marktplaats: Marktplaats,

    telegram: Telegram,
    offset: u64,
    telegram_poll_timeout_secs: u64,
    authorized_chat_ids: HashSet<i64>,

    command_builder: CommandBuilder,
}

#[bon]
impl Bot {
    #[builder(finish_fn = try_connect)]
    pub async fn new(
        db: Db,
        marktplaats: Marktplaats,
        telegram: Telegram,
        offset: u64,
        telegram_poll_timeout_secs: u64,
        authorized_chat_ids: HashSet<i64>,
    ) -> Result<Self> {
        let me = GetMe
            .call_on(&telegram)
            .await
            .context("failed to get bot’s user")?
            .username
            .context("the bot has no username")?;
        SetMyDescription::builder()
            .description("👋 This is a private bot for Marktplaats\n\nFeel free to set up your own instance from https://github.com/eigenein/mrktpltsbot")
            .build()
            .call_on(&telegram)
            .await
            .context("failed to set the bot description")?;
        let this = Self {
            db,
            marktplaats,
            telegram,
            offset,
            telegram_poll_timeout_secs,
            authorized_chat_ids,
            command_builder: CommandBuilder::new(&me)?,
        };
        Ok(this)
    }
}

impl Bot {
    /// Run the bot indefinitely.
    pub async fn run(self) {
        info!("Running Telegram bot…");
        telegram::updates()
            .telegram(self.telegram.clone())
            .offset(self.offset)
            .poll_timeout_secs(self.telegram_poll_timeout_secs)
            .build()
            .inspect_ok(|update| info!(update.id, "Received update"))
            .try_filter_map(|update| async { Ok(Option::<Message>::from(update)) })
            .inspect_ok(|message| info!(message.id, "Received message"))
            .try_filter_map(|message| async move {
                if let (Some(chat), Some(text)) = (message.chat, message.text) {
                    if let ChatId::Integer(chat_id) = chat.id {
                        Ok(Some((message.id, chat_id, text)))
                    } else {
                        Ok(None)
                    }
                } else {
                    warn!(message.id, "Message without an associated chat or text");
                    Ok(None)
                }
            })
            .inspect_ok(|(message_id, chat_id, text)| {
                info!(message_id, chat_id, text, "Filtered message");
            })
            .try_for_each(|(message_id, chat_id, text)| {
                let this = &self;
                async move {
                    if let Err(error) = this.on_message(chat_id, message_id, text.trim()).await {
                        this.on_error(chat_id, message_id, error).await;
                    }
                    info!(message_id, "Done");
                    Ok(())
                }
            })
            .await
            .expect("any error should be handled by now");
    }

    /// Gracefully handle the error.
    async fn on_error(&self, chat_id: i64, message_id: u64, error: Error) {
        error!("Failed to handle the update #{message_id}: {error:#}");
        let _ = SendMessage::builder()
            .chat_id(&ChatId::Integer(chat_id))
            .text("💥 An internal error occurred and has been logged")
            .build()
            .call_on(&self.telegram)
            .await;
    }

    #[instrument(skip_all, fields(chat_id = ?chat_id, message_id = message_id))]
    async fn on_message(&self, chat_id: i64, message_id: u64, text: &str) -> Result {
        if !self.authorized_chat_ids.contains(&chat_id) {
            warn!(chat_id, "Unauthorized");
            let chat_id = ChatId::Integer(chat_id);
            let text = render::unauthorized(&chat_id).render().into_string().into();
            let _ = SendMessage::quick_html(&chat_id, text)
                .call_on(&self.telegram)
                .await?;
            return Ok(());
        }

        let reply_parameters = ReplyParameters::builder()
            .message_id(message_id)
            .allow_sending_without_reply(true)
            .build();

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
                .parse_mode(ParseMode::Html)
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

    #[instrument(skip_all)]
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
                info!(chat_id, subscribe.query_hash, "Subscribing");
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
                let text = render::simple_message()
                    .markup("✅ You are now subscribed")
                    .links(&[unsubscribe_link])
                    .render();
                let _ = SendMessage::quick_html(&chat_id.into(), text.into())
                    .call_on(&self.telegram)
                    .await?;
            }

            if let Some(unsubscribe) = command.unsubscribe {
                // Unsubscribe from the search query.
                info!(chat_id, unsubscribe.query_hash, "Unsubscribing");
                let query_hash = QueryHash(unsubscribe.query_hash);
                let subscription = Subscription {
                    query_hash,
                    chat_id,
                };
                Subscriptions(&mut *self.db.connection().await)
                    .delete(&subscription)
                    .await?;
                let subscribe_link = self
                    .command_builder
                    .link()
                    .content("Re-subscribe")
                    .payload(
                        &CommandPayload::builder()
                            .subscribe(SubscriptionStartCommand::new(query_hash))
                            .build(),
                    )
                    .build();
                let text = render::simple_message()
                    .markup("✅ You are now unsubscribed")
                    .links(&[subscribe_link])
                    .render();
                let _ = SendMessage::quick_html(&chat_id.into(), text.into())
                    .call_on(&self.telegram)
                    .await?;
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
