use std::sync::atomic::{AtomicU64, Ordering};

use bon::Builder;

use crate::{
    db::{
        Db,
        search_query::{SearchQueries, SearchQuery},
    },
    marktplaats::{Marktplaats, SearchRequest},
    prelude::*,
    telegram::{
        Telegram,
        commands::{CommandBuilder, CommandPayload},
        methods::{AllowedUpdate, GetMe, GetUpdates, Method, SendMessage},
        notification::SendNotification,
        objects::{Chat, LinkPreviewOptions, ParseMode, ReplyParameters, Update, UpdatePayload},
        render,
    },
};

#[derive(Builder)] // TODO: async fallible builder that fetches `me` right away.
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
        let command_builder = CommandBuilder::new(&me)?;

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
                if let Err(error) = self.on_update(&command_builder, &update).await {
                    error!("Failed to handle the update #{}: {error:#}", update.id);
                }
            }
        }
    }

    #[instrument(skip_all, err(level = Level::DEBUG))]
    async fn on_update(&self, command_builder: &CommandBuilder, update: &Update) -> Result {
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
        self.on_message(command_builder, chat, text, reply_parameters)
            .await?; // TODO: inspect errors to user.

        info!(update.id, "Done");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn on_message(
        &self,
        command_builder: &CommandBuilder,
        chat: &Chat,
        text: &str,
        reply_parameters: ReplyParameters,
    ) -> Result {
        if text.starts_with('/') {
            self.handle_command(text, chat.id, reply_parameters).await
        } else {
            self.on_search(command_builder, text, chat.id, reply_parameters)
                .await
        }
    }

    /// Handle the search request from Telegram.
    ///
    /// A search request is just a message that is not a command.
    #[instrument(skip_all)]
    async fn on_search(
        &self,
        command_builder: &CommandBuilder,
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
        let command_payload = CommandPayload::subscribe_to(query.hash);
        let command_builder = command_builder.command().payload(&command_payload);

        if let Some(listing) = listings.inner.pop() {
            let description = render::listing_description()
                .listing(&listing)
                .search_query(&query)
                .links(&[command_builder.markup("Subscribe").build()])
                .render();
            SendNotification::builder()
                .chat_id(chat_id.into())
                .caption(&description)
                .pictures(&listing.pictures)
                .reply_parameters(reply_parameters)
                .build()
                .call_on(&self.telegram)
                .await?;
        } else {
            let text = render::simple_message()
                .markup("There is no item matching the search query")
                .links(&[command_builder.markup("Subscribe anyway").build()])
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
        chat_id: i64,
        reply_parameters: ReplyParameters,
    ) -> Result {
        Ok(())
    }
}
