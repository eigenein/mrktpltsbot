use std::time::Duration;

use serde::Serialize;

use crate::{
    client::DEFAULT_TIMEOUT,
    telegram::objects::{Update, User},
};

pub trait Request: Serialize {
    const METHOD_NAME: &'static str;

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

impl Request for GetMe {
    const METHOD_NAME: &'static str = "getMe";

    type Response = User;
}

/// [Update][1] types that the client wants to listen to.
///
/// [1]: https://core.telegram.org/bots/api#update
#[derive(Clone, Serialize, clap::ValueEnum)]
#[must_use]
pub enum AllowedUpdate {
    #[serde(rename = "message")]
    Message,
}

/// Use this method to receive incoming updates using long polling. Returns an `Array` of `Update` objects.
#[derive(Serialize)]
#[must_use]
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

impl Request for GetUpdates {
    const METHOD_NAME: &'static str = "getUpdates";

    type Response = Vec<Update>;

    fn timeout(&self) -> Duration {
        DEFAULT_TIMEOUT + Duration::from_secs(self.timeout_secs.unwrap_or_default())
    }
}
