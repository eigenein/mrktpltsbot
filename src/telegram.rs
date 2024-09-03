pub mod error;
pub mod methods;
pub mod objects;
pub mod result;

use std::fmt::Debug;

use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::{
    prelude::*,
    telegram::{error::TelegramError, methods::Method, result::TelegramResult},
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
    pub async fn call<R>(&self, request: R) -> Result<R::Response, TelegramError>
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
        serde_json::from_str::<TelegramResult<R::Response>>(&response)
            .with_context(|| format!("failed to deserialize `{}` response", R::NAME))?
            .into()
    }
}
