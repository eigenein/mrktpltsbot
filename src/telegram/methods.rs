use std::{borrow::Cow, fmt::Debug, time::Duration};

use bon::{Builder, builder};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    client::Client,
    prelude::*,
    serde::as_inner_json,
    telegram::{Telegram, objects::*},
};

/// Telegram bot API method.
pub trait Method: Serialize {
    type Response: Debug + DeserializeOwned;

    fn name(&self) -> &'static str;

    fn timeout(&self) -> Duration {
        Client::DEFAULT_TIMEOUT
    }

    /// Call the method on the specified Telegram connection.
    async fn call_on(&self, telegram: &Telegram) -> Result<Self::Response> {
        telegram.call(self).await
    }
}

/// A simple method for testing your bot's authentication token.
///
/// See also: <https://core.telegram.org/bots/api#getme>.
#[derive(Serialize)]
#[must_use]
pub struct GetMe;

impl Method for GetMe {
    type Response = User;

    fn name(&self) -> &'static str {
        "getMe"
    }
}

/// Use this method to change the bot's [description][1],
/// which is shown in the chat with the bot if the chat is empty.
///
/// [1]: https://core.telegram.org/bots/api#setmydescription
#[derive(Builder, Serialize)]
pub struct SetMyDescription<'a> {
    /// New bot description; 0-512 characters.
    /// Pass an empty string to remove the dedicated description for the given language.
    #[builder(into)]
    pub description: Option<Cow<'a, str>>,
}

impl<'a> Method for SetMyDescription<'a> {
    type Response = bool;

    fn name(&self) -> &'static str {
        "setMyDescription"
    }
}

/// [Update][1] types that the client wants to listen to.
///
/// [1]: https://core.telegram.org/bots/api#update
#[derive(Copy, Clone, Serialize, clap::ValueEnum)]
#[must_use]
pub enum AllowedUpdate {
    #[serde(rename = "message")]
    Message,
}

/// Use this method to receive incoming updates using long polling. Returns an `Array` of `Update` objects.
#[derive(Builder, Serialize)]
#[must_use]
pub struct GetUpdates<'a> {
    /// Identifier of the first update to be returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,

    /// Limits the number of updates to be retrieved. Values between 1-100 are accepted. Defaults to 100.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Timeout in seconds for long polling.
    ///
    /// Defaults to 0, i.e. usual short polling.
    /// Should be positive, short polling should be used for testing purposes only.
    #[serde(rename = "timeout", skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_updates: Option<&'a [AllowedUpdate]>,
}

impl<'a> Method for GetUpdates<'a> {
    type Response = Vec<Update>;

    fn name(&self) -> &'static str {
        "getUpdates"
    }

    fn timeout(&self) -> Duration {
        Client::DEFAULT_TIMEOUT + Duration::from_secs(self.timeout_secs.unwrap_or_default())
    }
}

/// [Send a message][1].
///
/// [1]: https://core.telegram.org/bots/api#sendmessage
#[derive(Builder, Serialize)]
#[must_use]
pub struct SendMessage<'a> {
    pub chat_id: &'a ChatId,

    #[builder(into)]
    pub text: Cow<'a, str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_preview_options: Option<LinkPreviewOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
}

impl Method for SendMessage<'_> {
    type Response = Message;

    fn name(&self) -> &'static str {
        "sendMessage"
    }
}

/// [Send a photo][1].
///
/// [1]: https://core.telegram.org/bots/api#sendphoto
#[derive(Builder, Serialize)]
#[must_use]
pub struct SendPhoto<'a> {
    pub chat_id: &'a ChatId,

    #[builder(into)]
    pub photo: Cow<'a, str>,

    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<Cow<'a, str>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_caption_above_media: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
}

impl Method for SendPhoto<'_> {
    type Response = Message;

    fn name(&self) -> &'static str {
        "sendPhoto"
    }
}

/// Use this method to [send a group][1] of photos, videos, documents or audios as an album.
///
/// [1]: https://core.telegram.org/bots/api#sendmediagroup
#[derive(Builder, Serialize)]
#[must_use]
pub struct SendMediaGroup<'a> {
    pub chat_id: &'a ChatId,

    /// A JSON-serialized array describing messages to be sent, must include 2-10 items.
    #[serde(serialize_with = "as_inner_json")]
    pub media: Vec<Media<'a>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
}

impl Method for SendMediaGroup<'_> {
    type Response = Vec<Message>;

    fn name(&self) -> &'static str {
        "sendMediaGroup"
    }
}

#[derive(Serialize)]
#[must_use]
#[serde(tag = "type")]
pub enum Media<'a> {
    #[serde(rename = "photo")]
    InputMediaPhoto(InputMediaPhoto<'a>),
}

#[derive(Builder, Serialize)]
#[must_use]
pub struct InputMediaPhoto<'a> {
    #[builder(into)]
    pub media: Cow<'a, str>,

    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<Cow<'a, str>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_caption_above_media: Option<bool>,
}
