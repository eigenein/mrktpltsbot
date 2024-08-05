pub mod methods;
pub mod objects;

use monostate::MustBe;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize};

use crate::{prelude::*, telegram::methods::Method};

#[must_use]
pub struct Telegram {
    client: Client,
    token: String,
}

impl Telegram {
    pub const fn new(client: Client, token: String) -> Self {
        Self { client, token }
    }

    #[instrument(skip_all, fields(method = R::NAME))]
    pub async fn call<R>(&self, request: R) -> Result<R::Response>
    where
        R: Method,
        R::Response: DeserializeOwned,
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
        ok: MustBe!(true),
        result: T,
    },

    Err {
        ok: MustBe!(false),
        description: String,
        error_code: i32,

        #[serde(default)]
        parameters: Option<ResponseParameters>,
    },
}

impl<T> From<Response<T>> for Result<T> {
    fn from(response: Response<T>) -> Self {
        match response {
            Response::Ok { result, .. } => Ok(result),
            Response::Err {
                description,
                error_code,
                ..
            } => Err(anyhow!("{description} ({error_code})")),
        }
    }
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
        assert_eq!(
            Result::from(serde_json::from_str::<Response<u32>>(
                r#"{"ok": true, "result": 42}"#
            )?)?,
            42
        );
        Ok(())
    }
}
