use monostate::MustBe;
use serde::Deserialize;

use crate::telegram::error::TelegramError;

/// Telegram bot API [response][1].
///
/// [1]: https://core.telegram.org/bots/api#making-requests
#[derive(Deserialize)]
#[must_use]
#[serde(untagged)]
pub enum TelegramResult<T> {
    Ok { ok: MustBe!(true), result: T },
    Err(TelegramError),
}

impl<T> From<TelegramResult<T>> for Result<T, TelegramError> {
    fn from(result: TelegramResult<T>) -> Self {
        match result {
            TelegramResult::Ok { result, .. } => Ok(result),
            TelegramResult::Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_ok() -> crate::prelude::Result {
        // language=json
        let response: TelegramResult<u32> = serde_json::from_str(r#"{"ok": true, "result": 42}"#)?;
        match response {
            TelegramResult::Ok { result, .. } => {
                assert_eq!(result, 42);
            }
            TelegramResult::Err(_) => unreachable!(),
        }
        Ok(())
    }

    #[test]
    fn test_too_many_requests() -> crate::prelude::Result {
        // language=json
        let response: TelegramResult<()> = serde_json::from_str(
            r#"{"ok": false, "error_code": 429, "description": "Too Many Requests: retry after X", "parameters": {"retry_after": 123}}"#,
        )?;
        match response {
            TelegramResult::Err(TelegramError::TooManyRequests { retry_after, .. }) => {
                assert_eq!(retry_after.secs, 123);
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}
