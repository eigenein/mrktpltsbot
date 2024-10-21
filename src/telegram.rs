pub mod error;
pub mod methods;
pub mod notification;
pub mod objects;
pub mod render;
pub mod result;
pub mod start;

use std::{fmt::Debug, time::Duration};

use backoff::{ExponentialBackoff, backoff::Backoff};
use serde::de::DeserializeOwned;
use tokio::time::sleep;
use url::Url;

use crate::{
    client::Client,
    prelude::*,
    telegram::{error::TelegramError, methods::Method, result::TelegramResult},
};

#[must_use]
pub struct Telegram {
    client: Client,
    token: String,
    root_url: Url,
}

impl Telegram {
    pub fn new(client: Client, token: String) -> Result<Self> {
        Ok(Self {
            client,
            token,
            root_url: Url::parse("https://api.telegram.org")?,
        })
    }

    /// Call the Telegram Bot API method with automatic throttling and retrying.
    #[instrument(skip_all, name = "call_telegram")]
    pub async fn call<R>(&self, request: &R) -> Result<R::Response>
    where
        R: Method + ?Sized,
        R::Response: Debug + DeserializeOwned,
    {
        let mut url = self.root_url.clone();
        url.set_path(&format!("bot{}/{}", self.token, request.name()));

        let request_builder = self
            .client
            .request(reqwest::Method::POST, url)
            .json(request)
            .timeout(request.timeout());

        let mut backoff = ExponentialBackoff::default();
        loop {
            let result = request_builder
                .try_clone()?
                .read_json::<TelegramResult<R::Response>>(false)
                .await;

            let error = match result {
                Ok(TelegramResult::Ok { result, .. }) => {
                    info!(name = request.name(), "Done");
                    break Ok(result);
                }

                Ok(TelegramResult::Err(TelegramError::TooManyRequests { retry_after, .. })) => {
                    warn!(name = request.name(), retry_after.secs, "Throttling");
                    sleep(Duration::from_secs(retry_after.secs)).await;
                    continue;
                }

                Ok(TelegramResult::Err(error)) => anyhow!("Telegram Bot API error: {error:#}"),

                Err(error) => error,
            };

            if let Some(duration) = backoff.next_backoff() {
                warn!(
                    name = request.name(),
                    ?duration,
                    "Retrying after the error: {error:#}",
                );
                sleep(duration).await;
            } else {
                warn!(name = request.name(), "All attempts have failed");
                break Err(error);
            }
        }
    }
}
