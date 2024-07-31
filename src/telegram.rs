use std::time::Duration;

use monostate::MustBe;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{client::DEFAULT_TIMEOUT, prelude::*};

#[must_use]
pub struct Telegram {
    client: Client,
    token: String,
}

impl Telegram {
    pub const fn new(client: Client, token: String) -> Self {
        Self { client, token }
    }

    #[instrument(skip_all, fields(method = R::METHOD_NAME))]
    pub async fn call<R>(&self, request: R) -> Result<R::Response>
    where
        R: Request,
        R::Response: DeserializeOwned,
    {
        let response = self
            .client
            .post(format!(
                "https://api.telegram.org/bot{}/{}",
                self.token,
                R::METHOD_NAME
            ))
            .json(&request)
            .timeout(request.timeout())
            .send()
            .await
            .with_context(|| format!("failed to call `{}`", R::METHOD_NAME))?
            .text()
            .await
            .with_context(|| format!("failed to read `{}` response", R::METHOD_NAME))?;
        debug!(response);
        serde_json::from_str::<Response<R::Response>>(&response)
            .with_context(|| format!("failed to deserialize `{}` response", R::METHOD_NAME))?
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
pub struct GetMe;

impl Request for GetMe {
    const METHOD_NAME: &'static str = "getMe";

    type Response = User;
}

/// This object represents a Telegram user or bot.
///
/// See also: <https://core.telegram.org/bots/api#user>.
#[derive(Deserialize)]
pub struct User {
    pub id: i64,

    #[serde(default)]
    pub username: Option<String>,
}
