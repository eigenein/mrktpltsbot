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

/// This object represents a [message][1].
///
/// [1]: https://core.telegram.org/bots/api#message
#[derive(Debug, Deserialize)]
#[must_use]
pub struct Message {
    #[serde(rename = "message_id")]
    pub id: u32,

    #[serde(default)]
    pub from: Option<User>,

    #[serde(default)]
    pub text: Option<String>,

    #[serde(default)]
    pub chat: Option<Chat>,
}

#[derive(Debug, Deserialize)]
#[must_use]
pub struct Chat {
    pub id: i64,
}
