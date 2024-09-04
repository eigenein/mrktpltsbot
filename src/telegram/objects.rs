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
