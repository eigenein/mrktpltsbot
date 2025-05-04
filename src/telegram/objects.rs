#![expect(dead_code)]

use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
};

use bon::Builder;
use serde::{Deserialize, Serialize};

/// This object represents a Telegram user or bot.
///
/// See also: <https://core.telegram.org/bots/api#user>.
#[derive(Debug, Deserialize)]
#[must_use]
pub struct User {
    pub id: i64,

    #[serde(default)]
    pub username: Option<String>,
}

// This object represents an incoming [update][1].
///
/// [1]: https://core.telegram.org/bots/api#update
#[derive(Debug, Deserialize)]
#[must_use]
pub struct Update {
    /// The update's unique identifier.
    ///
    /// Update identifiers start from a certain positive number and increase sequentially.
    #[serde(rename = "update_id")]
    pub id: u64,

    #[serde(flatten)]
    pub payload: UpdatePayload,
}

#[derive(Debug, Deserialize)]
#[must_use]
pub enum UpdatePayload {
    #[serde(rename = "message")]
    Message(Message),

    #[serde(other)]
    Other,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[must_use]
pub enum ChatId {
    Integer(i64),
    Username(String),
}

impl Display for ChatId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(chat_id) => chat_id.fmt(f),
            Self::Username(username) => username.fmt(f),
        }
    }
}

impl From<i64> for ChatId {
    fn from(chat_id: i64) -> Self {
        Self::Integer(chat_id)
    }
}

/// This object represents a [message][1].
///
/// [1]: https://core.telegram.org/bots/api#message
#[derive(Debug, Deserialize)]
#[must_use]
pub struct Message {
    #[serde(rename = "message_id")]
    pub id: u64,

    #[serde(default)]
    pub text: Option<String>,

    #[serde(default)]
    pub chat: Option<Chat>,
}

/// «Umbrella» for methods that may return exactly one [`Message`] or multiple messages.
///
/// For example, [`crate::telegram::methods::SendMessage`] and [`crate::telegram::methods::SendPhoto`]
/// return exactly one message, but [`crate::telegram::methods::SendMediaGroup`] returns multiple messages.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Messages {
    Single(Message),
    Multiple(Vec<Message>),
}

impl Messages {
    #[allow(clippy::missing_const_for_fn)]
    pub fn first(&self) -> Option<&Message> {
        match self {
            Self::Single(message) => Some(message),
            Self::Multiple(messages) => messages.first(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[must_use]
pub struct Chat {
    pub id: ChatId,
}

#[derive(Serialize)]
#[must_use]
pub enum ParseMode {
    /// [HTML style][1].
    ///
    /// [1]: https://core.telegram.org/bots/api#html-style
    #[serde(rename = "HTML")]
    Html,
}

/// Describes the [options][1] used for link preview generation.
///
/// [1]: https://core.telegram.org/bots/api#linkpreviewoptions
#[derive(Builder, Serialize)]
#[must_use]
pub struct LinkPreviewOptions {
    /// `true`, if the link preview is disabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_disabled: Option<bool>,

    /// URL to use for the link preview.
    ///
    /// If empty, then the first URL found in the message text will be used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// `true`, if the link preview must be shown above the message text;
    /// otherwise, the link preview will be shown below the message text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_above_text: Option<bool>,
}

impl LinkPreviewOptions {
    pub const DISABLED: Self = Self { is_disabled: Some(true), url: None, show_above_text: None };
}

/// Describes [reply parameters][1] for the message that is being sent.
///
/// [1]: https://core.telegram.org/bots/api#replyparameters
#[derive(Copy, Clone, Builder, Serialize)]
#[must_use]
pub struct ReplyParameters {
    /// Identifier of the message that will be replied to in the current chat,
    /// or in the chat `chat_id` if it is specified
    pub message_id: u64,

    /// Pass `true` if the message should be sent even if the specified message to be replied to is not found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_sending_without_reply: Option<bool>,
}

#[derive(Serialize)]
#[serde(untagged)]
#[must_use]
pub enum ReplyMarkup<'a> {
    InlineKeyboardMarkup(InlineKeyboardMarkup<'a>),
}

/// This object represents an [inline keyboard][1] that appears right next to the message it belongs to.
///
/// [1]: https://core.telegram.org/bots/api#inlinekeyboardmarkup
#[derive(Serialize)]
#[must_use]
pub struct InlineKeyboardMarkup<'a> {
    pub inline_keyboard: Vec<Vec<InlineKeyboardButton<'a>>>,
}

impl<'a> From<InlineKeyboardButton<'a>> for InlineKeyboardMarkup<'a> {
    fn from(button: InlineKeyboardButton<'a>) -> Self {
        Self { inline_keyboard: vec![vec![button]] }
    }
}

/// This object represents one [button of an inline keyboard][1].
///
/// [1]: https://core.telegram.org/bots/api#inlinekeyboardbutton
#[derive(Serialize)]
#[must_use]
pub struct InlineKeyboardButton<'a> {
    pub text: &'a str,

    #[serde(flatten)]
    pub action: InlineKeyboardButtonAction,
}

#[derive(Serialize)]
#[must_use]
pub enum InlineKeyboardButtonAction {
    #[serde(rename = "url")]
    Url(String),

    /// Data to be sent in a [callback query][1] to the bot when the button is pressed, 1-64 bytes.
    ///
    /// [1]: https://core.telegram.org/bots/api#callbackquery
    #[serde(rename = "callback_data")]
    CallbackData(String),
}

/// This object represents a [bot command][1].
///
/// [1]: https://core.telegram.org/bots/api#botcommand
#[derive(Builder, Serialize)]
#[must_use]
pub struct BotCommand<'a> {
    /// Text of the command; 1-32 characters.
    /// Can contain only lowercase English letters, digits and underscores.
    pub command: &'a str,

    /// Description of the command; 1-256 characters.
    pub description: &'a str,
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
