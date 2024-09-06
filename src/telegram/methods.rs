use std::{borrow::Cow, fmt::Debug, time::Duration};

use bon::builder;
use serde::{de::DeserializeOwned, Serialize, Serializer};

use crate::{
    client::DEFAULT_TIMEOUT,
    prelude::*,
    telegram::{error::TelegramError, objects::*, Telegram},
};

/// Telegram bot API method.
pub trait Method: Serialize {
    /// Method name.
    const NAME: &'static str;

    type Response: Debug + DeserializeOwned;

    fn timeout(&self) -> Duration {
        DEFAULT_TIMEOUT
    }

    async fn call_on(&self, telegram: &Telegram) -> Result<Self::Response, TelegramError> {
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
    const NAME: &'static str = "getMe";

    type Response = User;
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
#[derive(Serialize)]
#[must_use]
#[builder]
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
    const NAME: &'static str = "getUpdates";

    type Response = Vec<Update>;

    fn timeout(&self) -> Duration {
        DEFAULT_TIMEOUT + Duration::from_secs(self.timeout_secs.unwrap_or_default())
    }
}

/// [Send a message][1].
///
/// [1]: https://core.telegram.org/bots/api#sendmessage
#[derive(Serialize)]
#[must_use]
#[builder]
pub struct SendMessage<'a> {
    #[builder(into)]
    pub chat_id: ChatId,

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
    const NAME: &'static str = "sendMessage";
    type Response = Message;
}

/// [Send a photo][1].
///
/// [1]: https://core.telegram.org/bots/api#sendphoto
#[derive(Serialize)]
#[must_use]
#[builder]
pub struct SendPhoto<'a> {
    pub chat_id: ChatId,

    #[builder(into)]
    pub photo: Cow<'a, str>,

    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<Cow<'a, str>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_caption_above_media: Option<bool>,
}

impl Method for SendPhoto<'_> {
    const NAME: &'static str = "sendPhoto";
    type Response = Message;
}

fn serialize_media<S: Serializer>(media: &[Media], serializer: S) -> Result<S::Ok, S::Error> {
    let json = serde_json::to_string(media)
        .map_err(|error| serde::ser::Error::custom(format!("{error:#}")))?;
    serializer.serialize_str(&json)
}

/// Use this method to [send a group][1] of photos, videos, documents or audios as an album.
///
/// [1]: https://core.telegram.org/bots/api#sendmediagroup
#[derive(Serialize)]
#[must_use]
#[builder]
pub struct SendMediaGroup<'a> {
    pub chat_id: ChatId,

    /// A JSON-serialized array describing messages to be sent, must include 2-10 items.
    #[serde(serialize_with = "serialize_media")]
    pub media: Vec<Media<'a>>,
}

impl Method for SendMediaGroup<'_> {
    const NAME: &'static str = "sendMediaGroup";
    type Response = Vec<Message>;
}

#[derive(Serialize)]
#[must_use]
#[serde(tag = "type")]
pub enum Media<'a> {
    #[serde(rename = "photo")]
    InputMediaPhoto(InputMediaPhoto<'a>),
}

#[derive(Serialize)]
#[must_use]
#[builder]
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
