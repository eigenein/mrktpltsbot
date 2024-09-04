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
}

#[derive(Debug, Deserialize)]
#[must_use]
pub struct Chat {
    #[allow(dead_code)]
    pub id: i64,
}
