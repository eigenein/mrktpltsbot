use std::borrow::Cow;

use bon::bon;
use serde::Serialize;
use url::Url;

use crate::{
    prelude::*,
    telegram::{
        Telegram,
        methods::{Method, SendMessage, SendPhoto},
        objects::{ChatId, LinkPreviewOptions, ParseMode, ReplyParameters},
    },
};

/// Reaction method on Telegram.
#[derive(Serialize)]
#[serde(untagged)]
#[must_use]
pub enum Notification<'a> {
    Message(SendMessage<'a>),
    Photo(SendPhoto<'a>),
}

#[bon]
impl<'a> Notification<'a> {
    /// Build a new reaction method from a listing contents.
    #[builder]
    pub fn new(
        chat_id: Cow<'a, ChatId>,
        text: Cow<'a, str>,
        parse_mode: ParseMode,
        picture_url: Option<&'a Url>,
        reply_parameters: Option<ReplyParameters>,
    ) -> Self {
        // Specific representation depends on how many pictures there are.
        match picture_url {
            None => Self::Message(
                SendMessage::builder()
                    .chat_id(chat_id)
                    .text(text)
                    .parse_mode(parse_mode)
                    .link_preview_options(LinkPreviewOptions::DISABLED)
                    .maybe_reply_parameters(reply_parameters)
                    .build(),
            ),

            Some(url) => Self::Photo(
                SendPhoto::builder()
                    .chat_id(chat_id)
                    .photo(url.as_str())
                    .caption(text)
                    .parse_mode(parse_mode)
                    .maybe_reply_parameters(reply_parameters)
                    .build(),
            ),
        }
    }
}

impl Notification<'_> {
    pub async fn send_to(&self, telegram: &Telegram) -> Result {
        match self {
            Notification::Message(inner) => inner.call_and_discard_on(telegram).await,
            Notification::Photo(inner) => inner.call_and_discard_on(telegram).await,
        }
    }
}
