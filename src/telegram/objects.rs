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
    pub id: u32,

    #[serde(flatten)]
    pub payload: UpdatePayload,
}

#[derive(Debug, Deserialize)]
#[must_use]
pub enum UpdatePayload {
    #[serde(rename = "message")]
    #[allow(dead_code)]
    Message(Message),

    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
#[must_use]
pub enum ChatId {
    #[allow(dead_code)]
    Integer(i64),

    #[allow(dead_code)]
    Username(String),
}

/// This object represents a [message][1].
///
/// [1]: https://core.telegram.org/bots/api#message
#[derive(Debug, Deserialize)]
#[must_use]
pub struct Message {
    #[serde(rename = "message_id")]
    #[allow(dead_code)]
    pub id: u32,

    #[serde(default)]
    #[allow(dead_code)]
    pub from: Option<User>,

    #[serde(default)]
    #[allow(dead_code)]
    pub text: Option<String>,

    #[serde(default)]
    #[allow(dead_code)]
    pub chat: Option<Chat>,

    #[serde(default)]
    #[allow(dead_code)]
    pub entities: Vec<MessageEntity>,
}

/// This object represents one [special entity][1] in a text message.
///
/// [1]: https://core.telegram.org/bots/api#messageentity
#[derive(Debug, Deserialize)]
#[must_use]
pub struct MessageEntity {
    #[serde(default)]
    #[allow(dead_code)]
    pub custom_emoji_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[must_use]
pub struct Chat {
    #[allow(dead_code)]
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
#[derive(Default, Serialize)]
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

#[derive(Serialize)]
#[must_use]
#[serde(untagged)]
pub enum ReplyMarkup<'a> {
    #[allow(dead_code)]
    InlineKeyboard(InlineKeyboardMarkup<'a>),
}

impl<'a> From<InlineKeyboardMarkup<'a>> for ReplyMarkup<'a> {
    fn from(value: InlineKeyboardMarkup<'a>) -> Self {
        Self::InlineKeyboard(value)
    }
}

/// This object represents an [inline keyboard][1] that appears right next to the message it belongs to.
///
/// [1]: https://core.telegram.org/bots/api#inlinekeyboardmarkup
#[must_use]
#[derive(Serialize)]
pub struct InlineKeyboardMarkup<'a> {
    pub inline_keyboard: Vec<Vec<InlineKeyboardButton<'a>>>,
}

impl<'a> InlineKeyboardMarkup<'a> {
    pub fn single_button(button: InlineKeyboardButton<'a>) -> Self {
        Self {
            inline_keyboard: vec![vec![button]],
        }
    }
}

/// This object represents [one button of an inline keyboard][1].
///
/// [1]: https://core.telegram.org/bots/api#inlinekeyboardbutton
#[must_use]
#[derive(Serialize)]
pub struct InlineKeyboardButton<'a> {
    /// Label text on the button.
    pub text: &'a str,

    #[serde(flatten)]
    pub payload: InlineKeyboardButtonPayload<'a>,
}

#[must_use]
#[derive(Serialize)]
pub enum InlineKeyboardButtonPayload<'a> {
    /// HTTP or `tg://` URL to be opened when the button is pressed.
    ///
    /// Links `tg://user?id=<user_id>` can be used to mention a user by their identifier
    /// without using a username, if this is allowed by their privacy settings.
    #[serde(rename = "url")]
    Url(&'a str),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{prelude::*, telegram::methods::SendMessage};

    #[test]
    fn message_with_inline_keyboard_ok() -> Result {
        let inline_keyboard_markup = InlineKeyboardMarkup::single_button(InlineKeyboardButton {
            text: "Test",
            payload: InlineKeyboardButtonPayload::Url("https://example.org"),
        });
        let send_message = SendMessage::builder()
            .chat_id(ChatId::Integer(42))
            .text("test")
            .reply_markup(ReplyMarkup::InlineKeyboard(inline_keyboard_markup))
            .build();
        assert_eq!(
            serde_json::to_string(&send_message)?,
            // language=json
            r#"{"chat_id":42,"text":"test","reply_markup":"{\"inline_keyboard\":[[{\"text\":\"Test\",\"url\":\"https://example.org\"}]]}"}"#,
        );
        Ok(())
    }
}
