pub mod error;
pub mod methods;
pub mod notification;
pub mod objects;
pub mod render;
pub mod result;
pub mod start;

use std::{fmt::Debug, time::Duration};

use reqwest::Client;
use serde::de::DeserializeOwned;
use tokio::time::sleep;

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

    #[instrument(skip_all, fields(method = request.name()), ret(level = Level::DEBUG), err(level = Level::DEBUG))]
    pub async fn call<R>(&self, request: &R) -> Result<R::Response, TelegramError>
    where
        R: Method + ?Sized,
        R::Response: Debug + DeserializeOwned,
    {
        let url = format!(
            "https://api.telegram.org/bot{}/{}",
            self.token,
            request.name()
        );
        loop {
            let response = self
                .client
                .post(&url)
                .json(&request)
                .timeout(request.timeout())
                .send()
                .await
                .with_context(|| format!("failed to call `{}`", request.name()))?
                .text()
                .await
                .with_context(|| format!("failed to read `{}` response", request.name()))?;
            trace!(response, "Got raw response");
            let result = serde_json::from_str::<TelegramResult<R::Response>>(&response)
                .with_context(|| format!("failed to deserialize `{}` response", request.name()))?;
            match result {
                TelegramResult::Err(TelegramError::TooManyRequests { retry_after, .. }) => {
                    sleep(Duration::from_secs(retry_after.secs)).await;
                }
                _ => break result.into(),
            }
        }
    }
}
