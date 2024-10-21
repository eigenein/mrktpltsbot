use monostate::MustBe;
use serde::Deserialize;

use crate::prelude::*;

/// Telegram bot API [error][1].
///
/// [1]: https://core.telegram.org/bots/api#making-requests
#[derive(Debug, Deserialize, thiserror::Error)]
#[must_use]
#[serde(untagged)]
pub enum TelegramError {
    #[error("too many requests, retry after {} secs", retry_after.secs)]
    TooManyRequests {
        ok: MustBe!(false),

        error_code: MustBe!(429),

        #[serde(rename = "parameters")]
        retry_after: RetryAfterParameters,
    },

    #[error("API error ({error_code}) {description}")]
    ApiError {
        ok: MustBe!(false),

        description: String,
        error_code: i32,
    },
}

/// [Additional error details for exceeded rate limit][1].
///
/// [1]: https://core.telegram.org/bots/api#responseparameters
#[derive(Debug, Deserialize)]
pub struct RetryAfterParameters {
    #[serde(rename = "retry_after")]
    pub secs: u64,
}
