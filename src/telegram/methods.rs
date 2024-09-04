use std::time::Duration;

use bon::builder;
use serde::{Serialize, Serializer};

use crate::{
    client::DEFAULT_TIMEOUT,
    telegram::objects::{
        ChatId,
        LinkPreviewOptions,
        Message,
        ParseMode,
        ReplyMarkup,
        Update,
        User,
    },
};

/// Telegram bot API method.
pub trait Method: Serialize {
    /// Method name.
    const NAME: &'static str;

    type Response;

    fn timeout(&self) -> Duration {
        DEFAULT_TIMEOUT
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
pub struct GetUpdates {
    /// Identifier of the first update to be returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,

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
    pub allowed_updates: Option<Vec<AllowedUpdate>>,
}

impl Method for GetUpdates {
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
    pub chat_id: ChatId,
    pub text: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_preview_options: Option<LinkPreviewOptions>,

    #[serde(
        serialize_with = "serialize_reply_markup",
        skip_serializing_if = "Option::is_none"
    )]
    #[builder(into)]
    pub reply_markup: Option<ReplyMarkup<'a>>,
}

impl Method for SendMessage<'_> {
    const NAME: &'static str = "sendMessage";
    type Response = Message;
}

fn serialize_reply_markup<S: Serializer>(
    reply_markup: &Option<ReplyMarkup>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let reply_markup = reply_markup
        .as_ref()
        .expect("`reply_markup` should not be `None`");
    let json = serde_json::to_string(&reply_markup)
        .map_err(|error| serde::ser::Error::custom(format!("{error:#}")))?;
    serializer.serialize_str(&json)
}
