pub mod error;
pub mod methods;
pub mod notification;
pub mod objects;
pub mod render;
pub mod result;
pub mod start;

use std::{fmt::Debug, time::Duration};

use backoff::{ExponentialBackoff, backoff::Backoff};
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

    #[instrument(skip_all, ret(level = Level::DEBUG), err(level = Level::DEBUG))]
    async fn call_once<R>(&self, request: &R) -> Result<R::Response, TelegramError>
    where
        R: Method + ?Sized,
        R::Response: Debug + DeserializeOwned,
    {
        let method_name = request.name();
        let url = format!(
            "https://api.telegram.org/bot{}/{}",
            self.token,
            request.name()
        ); // TODO: build URL once.
        let response = self
            .client
            .post(&url)
            .json(&request) // TODO: serialize once.
            .timeout(request.timeout())
            .send()
            .await
            .with_context(|| format!("failed to call `{method_name}`"))?
            .text()
            .await
            .with_context(|| format!("failed to read `{method_name}` response"))?;
        trace!(response, "Received response"); // TODO: proper tracing.
        serde_json::from_str::<TelegramResult<R::Response>>(&response)
            .with_context(|| format!("failed to deserialize `{method_name}` response"))?
            .into()
    }

    #[instrument(skip_all, fields(method = request.name()))]
    pub async fn call<R>(&self, request: &R) -> Result<R::Response, TelegramError>
    where
        R: Method + ?Sized,
        R::Response: Debug + DeserializeOwned,
    {
        let mut backoff = ExponentialBackoff::default();
        loop {
            match self.call_once(request).await {
                // Success:
                result @ Ok(_) => {
                    info!("Ok");
                    break result;
                }

                // Rate limit exceeded:
                Err(TelegramError::TooManyRequests { retry_after, .. }) => {
                    warn!(retry_after.secs, "Throttling");
                    sleep(Duration::from_secs(retry_after.secs)).await;
                }

                // Unexpected error:
                Err(error) => {
                    if let Some(duration) = backoff.next_backoff() {
                        warn!(?duration, "Retrying after the error: {error:#}");
                        sleep(duration).await;
                    } else {
                        warn!("All attempts have failed");
                        break Err(error);
                    }
                }
            }
        }
    }
}
