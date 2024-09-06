use std::sync::atomic::{AtomicU64, Ordering};

use backoff::ExponentialBackoff;
use bon::builder;

use crate::{
    db::Db,
    marktplaats::Marktplaats,
    prelude::*,
    telegram::{
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
            backoff::future::retry(ExponentialBackoff::default(), || async {
                Ok(self.handle_telegram_updates().await?)
            })
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

        if text.starts_with('/') {
            let _ = SendMessage::builder()
                .chat_id(chat.id)
                .text("I can't answer commands just yet")
                .reply_parameters(
                    ReplyParameters::builder()
                        .message_id(message.id)
                        .allow_sending_without_reply(true)
                        .build(),
                )
                .build()
                .call_on(&self.telegram)
                .await?;
        } else {
            todo!()
        }

        Ok(())
    }
}
