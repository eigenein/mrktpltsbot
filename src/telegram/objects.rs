#![expect(dead_code)]

use bon::builder;
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

#[derive(Debug, Serialize)]
#[serde(untagged)]
#[must_use]
pub enum ChatId {
    Integer(i64),

    Username(String),
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
    #[expect(dead_code)]
    pub from: Option<User>,

    #[serde(default)]
    pub text: Option<String>,

    #[serde(default)]
    pub chat: Option<Chat>,

    #[serde(default)]
    #[expect(dead_code)]
    pub entities: Vec<MessageEntity>,
}

/// This object represents one [special entity][1] in a text message.
///
/// [1]: https://core.telegram.org/bots/api#messageentity
#[derive(Debug, Deserialize)]
#[must_use]
pub struct MessageEntity {
    #[serde(default)]
    #[expect(dead_code)]
    pub custom_emoji_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[must_use]
pub struct Chat {
    pub id: i64,
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
#[derive(Serialize)]
#[must_use]
#[builder]
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

/// Describes [reply parameters][1] for the message that is being sent.
///
/// [1]: https://core.telegram.org/bots/api#replyparameters
#[derive(Serialize)]
#[must_use]
#[builder]
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
        Self {
            inline_keyboard: vec![vec![button]],
        }
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
