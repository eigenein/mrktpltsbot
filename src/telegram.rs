pub mod error;
pub mod methods;
pub mod objects;
pub mod result;

use std::fmt::Debug;

use monostate::MustBe;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize};

use crate::{
    prelude::*,
    telegram::{methods::Method, result::TelegramResult},
};

#[must_use]
pub struct Telegram {
    client: Client,
    token: String,
}

impl Telegram {
    pub const fn new(client: Client, token: String) -> Self {
        Self { client, token }
    }

    #[instrument(skip_all, fields(method = R::NAME), ret(level = Level::DEBUG), err(level = Level::DEBUG))]
    pub async fn call<R>(&self, request: R) -> TelegramResult<R::Response>
    where
        R: Method,
        R::Response: Debug + DeserializeOwned,
    {
        let response = self
            .client
            .post(format!(
                "https://api.telegram.org/bot{}/{}",
                self.token,
                R::NAME
            ))
            .json(&request)
            .timeout(request.timeout())
            .send()
            .await
            .with_context(|| format!("failed to call `{}`", R::NAME))?
            .text()
            .await
            .with_context(|| format!("failed to read `{}` response", R::NAME))?;
        debug!(response);
        serde_json::from_str::<Response<R::Response>>(&response)
            .with_context(|| format!("failed to deserialize `{}` response", R::NAME))?
            .into()
    }
}

/// Telegram bot API [response][1].
///
/// [1]: https://core.telegram.org/bots/api#making-requests
#[derive(Deserialize)]
#[must_use]
#[serde(untagged)]
enum Response<T> {
    Ok {
        #[allow(dead_code)]
        ok: MustBe!(true),

        result: T,
    },

    Err {
        #[allow(dead_code)]
        ok: MustBe!(false),

        description: String,
        error_code: i32,

        #[serde(default)]
        parameters: Option<ResponseParameters>,
    },
}

/// [Response parameters][1].
///
/// [1]: https://core.telegram.org/bots/api#responseparameters
#[derive(Deserialize)]
pub struct ResponseParameters {
    #[serde(rename = "retry_after", default)]
    pub retry_after_secs: Option<u32>,
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

    #[test]
    fn test_too_many_requests() -> Result {
        // language=json
        let response: Response<()> = serde_json::from_str(
            r#"{"ok": false, "error_code": 429, "description": "Too Many Requests: retry after X", "parameters": {"retry_after": 123}}"#,
        )?;
        match response {
            Response::Err {
                error_code,
                parameters,
                ..
            } => {
                assert_eq!(error_code, 429);
                assert_eq!(parameters.unwrap().retry_after_secs.unwrap(), 123);
            }
            Response::Ok { .. } => unreachable!(),
        }
        Ok(())
    }
}
