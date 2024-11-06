use std::{borrow::Cow, collections::HashSet};

use bon::Builder;
use futures::{Stream, TryStreamExt};
use maud::Render;

use crate::{
    db::{
        Db,
        search_query::{SearchQueries, SearchQuery},
        subscription::{Subscription, Subscriptions},
    },
    marktplaats::{Marktplaats, SearchRequest},
    prelude::*,
    telegram::{
        Telegram,
        commands::{CommandBuilder, CommandPayload, SubscriptionAction},
        methods::{GetMe, Method, SendMessage, SetMyDescription},
        objects::{ChatId, LinkPreviewOptions, Message, ParseMode, ReplyParameters, Update},
        reaction::Reaction,
        render,
    },
};

/// Telegram [`Message`] reactor.
///
/// It listens to Telegram [`Update`]'s and produces reactions on them.
#[derive(Builder)]
pub struct Reactor<'s> {
    authorized_chat_ids: HashSet<i64>,
    db: &'s Db,
    marktplaats: &'s Marktplaats,
    command_builder: &'s CommandBuilder,
}

impl<'s> Reactor<'s> {
    /// Run the reactor indefinitely and produce reactions.
    pub fn run(
        &'s self,
        updates: impl Stream<Item = Result<Update>> + 's,
    ) -> impl Stream<Item = Result<Vec<Reaction<'static>>>> + 's {
        info!(
            me = self.command_builder.url().as_str(),
            "Running Telegram reactor…",
        );
        updates
            .inspect_ok(|update| info!(update.id, "Received update"))
            .try_filter_map(|update| async { Ok(Option::<Message>::from(update)) })
            .inspect_ok(|message| debug!(message.id, "Received message"))
            .try_filter_map(|message| async move {
                // TODO: extract `filter_message`?
                if let (Some(chat), Some(text)) = (message.chat, message.text) {
                    if let ChatId::Integer(chat_id) = chat.id {
                        Ok(Some((message.id, chat_id, text)))
                    } else {
                        warn!(message.id, "Username chat IDs are not supported");
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
            .and_then(move |(message_id, chat_id, text)| async move {
                match self.on_message(chat_id, message_id, text.trim()).await {
                    Ok(reactions) => {
                        info!(chat_id, message_id, "Done");
                        Ok(reactions)
                    }
                    Err(error) => Ok(vec![Self::on_error(chat_id.into(), message_id, &error)]),
                }
            })
    }

    /// Gracefully handle the error.
    #[instrument(skip_all, fields(chat_id = %chat_id, message_id = message_id))]
    fn on_error(chat_id: ChatId, message_id: u64, error: &Error) -> Reaction<'static> {
        error!("Failed to handle the message: {error:#}");
        SendMessage::builder()
            .chat_id(Cow::Owned(chat_id))
            .text("💥 An internal error occurred and has been logged")
            .build()
            .into()
    }

    #[instrument(skip_all, fields(chat_id = chat_id, message_id = message_id, text = text))]
    async fn on_message(
        &self,
        chat_id: i64,
        message_id: u64,
        text: &str,
    ) -> Result<Vec<Reaction<'static>>> {
        if !self.authorized_chat_ids.contains(&chat_id) {
            warn!("Unauthorized");
            let chat_id = ChatId::Integer(chat_id);
            let text = render::unauthorized(&chat_id).render().into_string().into();
            return Ok(vec![
                SendMessage::quick_html(Cow::Owned(chat_id), text).into(),
            ]);
        }

        let reply_parameters = ReplyParameters::builder()
            .message_id(message_id)
            .allow_sending_without_reply(true)
            .build();

        if text.starts_with('/') {
            self.on_command(text, chat_id, reply_parameters).await
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
    ) -> Result<Vec<Reaction<'static>>> {
        let query = SearchQuery::from(query);
        let request = SearchRequest::standard(&query.text, 1);
        let mut listings = self.marktplaats.search(&request).await?;
        info!(query.hash, n_listings = listings.inner.len());

        SearchQueries(&mut *self.db.connection().await)
            .upsert(&query)
            .await?;

        // We need the subscribe command anyway, even if no listings were found.
        let subscribe_link = self
            .command_builder
            .link()
            .payload(&CommandPayload::subscribe_to(query.hash))
            .content("Subscribe")
            .build();

        if let Some(listing) = listings.inner.pop() {
            let description = render::listing_description()
                .listing(&listing)
                .search_query(&query)
                .links(&[subscribe_link])
                .render();
            Ok(vec![
                Reaction::from_listing()
                    .chat_id(Cow::Owned(chat_id.into()))
                    .text(description)
                    .pictures(listing.pictures)
                    .reply_parameters(reply_parameters)
                    .parse_mode(ParseMode::Html)
                    .build(),
            ])
        } else {
            let text = render::simple_message()
                .markup("There are no items matching the search query. Try a different query or subscribe anyway to wait for them to appear")
                .links(&[subscribe_link])
                .render();
            Ok(vec![
                SendMessage::builder()
                    .chat_id(Cow::Owned(chat_id.into()))
                    .text(text)
                    .parse_mode(ParseMode::Html)
                    .reply_parameters(reply_parameters)
                    .link_preview_options(LinkPreviewOptions::DISABLED)
                    .build()
                    .into(),
            ])
        }
    }

    #[instrument(skip_all)]
    async fn on_command(
        &self,
        text: &str,
        chat_id: i64,
        reply_parameters: ReplyParameters,
    ) -> Result<Vec<Reaction<'static>>> {
        if text == "/start" {
            // Just an initial greeting.
            let chat_id: Cow<'_, ChatId> = Cow::Owned(ChatId::Integer(chat_id));
            return Ok(vec![
                SendMessage::builder()
                    .chat_id(chat_id.clone())
                    .text("👋")
                    .build()
                    .into(),
                SendMessage::builder()
                    .chat_id(chat_id)
                    .text("Just send me a search query to start")
                    .build()
                    .into(),
            ]);
        }

        if let Some(payload) = text.strip_prefix("/start ") {
            // Command with a payload.
            let command = CommandPayload::from_base64(payload)?;
            debug!(?command, "Received command");
            let mut reactions = Vec::new();

            if let Some(subscription_command) = command.subscription {
                let subscription = Subscription {
                    query_hash: subscription_command.query_hash,
                    chat_id,
                };
                let connection = &mut *self.db.connection().await;
                let mut subscriptions = Subscriptions(connection);

                match SubscriptionAction::try_from(subscription_command.action) {
                    Ok(SubscriptionAction::Subscribe) => {
                        info!(subscription.query_hash, "Subscribing");
                        subscriptions.upsert(&subscription).await?;
                        let unsubscribe_link = self
                            .command_builder
                            .link()
                            .content("Unsubscribe")
                            .payload(&CommandPayload::unsubscribe_from(subscription.query_hash))
                            .build();
                        let text = render::simple_message()
                            .markup("You are now subscribed")
                            .links(&[unsubscribe_link])
                            .render();
                        reactions.push(
                            SendMessage::quick_html(Cow::Owned(chat_id.into()), text.into()).into(),
                        );
                    }

                    Ok(SubscriptionAction::Unsubscribe) => {
                        info!(subscription.query_hash, "Unsubscribing");
                        subscriptions.delete(&subscription).await?;
                        let subscribe_link = self
                            .command_builder
                            .link()
                            .content("Re-subscribe")
                            .payload(&CommandPayload::subscribe_to(subscription.query_hash))
                            .build();
                        let text = render::simple_message()
                            .markup("You are now unsubscribed")
                            .links(&[subscribe_link])
                            .render();
                        reactions.push(
                            SendMessage::quick_html(Cow::Owned(chat_id.into()), text.into()).into(),
                        );
                    }

                    _ => {} // TODO: technically, I should return a message that the action is no longer supported
                }
            }

            return Ok(reactions);
        }

        // Unknown command.
        Ok(vec![
            SendMessage::builder()
                .chat_id(Cow::Owned(chat_id.into()))
                .text("I am sorry, but I do not know this command")
                .reply_parameters(reply_parameters)
                .build()
                .into(),
        ])
    }
}

/// Initialize the Telegram bot.
#[instrument(skip_all)]
pub async fn try_init(telegram: &Telegram) -> Result<CommandBuilder> {
    let me = GetMe
        .call_on(telegram)
        .await
        .context("failed to get bot’s user")?
        .username
        .context("the bot has no username")?;
    SetMyDescription::builder()
        .description("👋 This is a private bot for Marktplaats\n\nFeel free to set up your own instance from https://github.com/eigenein/mrktpltsbot")
        .build()
        .call_on(telegram)
        .await
        .context("failed to set the bot description")?;
    CommandBuilder::new(&me)
}
