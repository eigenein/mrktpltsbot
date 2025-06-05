use monostate::MustBe;
use serde::Deserialize;

use crate::prelude::*;

/// Telegram bot API [response][1].
///
/// [1]: https://core.telegram.org/bots/api#making-requests
#[derive(Deserialize)]
#[must_use]
#[serde(untagged)]
pub enum Response<T> {
    Ok { ok: MustBe!(true), result: T },
    Err { ok: MustBe!(false), description: String, error_code: i32 },
}

impl<T> From<Response<T>> for Result<T> {
    fn from(result: Response<T>) -> Self {
        match result {
            Response::Ok { result, .. } => Ok(result),
            Response::Err { error_code, description, .. } => {
                Err(anyhow!("API error {error_code}: {description}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_ok() -> Result {
        // language=json
        let response: Response<u32> = serde_json::from_str(r#"{"ok": true, "result": 42}"#)?;
        match response {
            Response::Ok { result, .. } => {
                assert_eq!(result, 42);
            }
            Response::Err { .. } => unreachable!(),
        }
        Ok(())
    }
}
