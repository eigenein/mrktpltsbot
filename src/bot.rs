use std::{
    collections::HashSet,
    sync::atomic::{AtomicU64, Ordering},
};

use bon::bon;
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
        commands::{CommandBuilder, CommandPayload, SubscriptionStartCommand},
        methods::{AllowedUpdate, GetMe, GetUpdates, Method, SendMessage, SetMyDescription},
        notification::SendNotification,
        objects::{
            Chat,
            ChatId,
            LinkPreviewOptions,
            ParseMode,
            ReplyParameters,
            Update,
            UpdatePayload,
        },
        render,
        render::unauthorized,
    },
};

pub struct Bot {
    telegram: Telegram,
    authorized_chat_ids: HashSet<ChatId>,
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
        authorized_chat_ids: HashSet<ChatId>,
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
                if let Err(error) = self.on_update(&update).await {
                    error!("Failed to handle the update #{}: {error:#}", update.id);
                }
            }
        }
    }

    #[instrument(skip_all, err(level = Level::DEBUG))]
    async fn on_update(&self, update: &Update) -> Result {
        let UpdatePayload::Message(message) = &update.payload else {
            bail!("The bot should only receive message updates")
        };
        let (Some(chat), Some(text)) = (&message.chat, &message.text) else {
            bail!("Message without an associated chat or text");
        };
        info!(update.id, ?chat.id, text, "Received");

        let reply_parameters = ReplyParameters::builder()
            .message_id(message.id)
            .allow_sending_without_reply(true)
            .build();
        self.on_message(chat, text, reply_parameters).await?; // TODO: inspect errors to user.

        info!(update.id, "Done");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn on_message(
        &self,
        chat: &Chat,
        text: &str,
        reply_parameters: ReplyParameters,
    ) -> Result {
        if !self.authorized_chat_ids.contains(&chat.id) {
            warn!(?chat.id, "Unauthorized");
            let _ = SendMessage::builder()
                .chat_id(&chat.id)
                .text(unauthorized(&chat.id).render().into_string())
                .parse_mode(ParseMode::Html)
                .link_preview_options(LinkPreviewOptions::DISABLED)
                .build()
                .call_on(&self.telegram)
                .await?;
            Ok(())
        } else if text.starts_with('/') {
            self.handle_command(text, &chat.id, reply_parameters).await
        } else {
            self.on_search(text, &chat.id, reply_parameters).await
        }
    }

    /// Handle the search request from Telegram.
    ///
    /// A search request is just a message that is not a command.
    #[instrument(skip_all)]
    async fn on_search(
        &self,
        query: &str,
        chat_id: &ChatId,
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
                .chat_id(chat_id)
                .caption(&description)
                .pictures(&listing.pictures)
                .reply_parameters(reply_parameters)
                .build()
                .call_on(&self.telegram)
                .await?;
        } else {
            let text = render::simple_message()
                .markup("There are no items matching the search query")
                .links(&[subscribe_link])
                .render();
            let _ = SendMessage::builder()
                .chat_id(chat_id)
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
        chat_id: &ChatId,
        reply_parameters: ReplyParameters,
    ) -> Result {
        Ok(())
    }
}
