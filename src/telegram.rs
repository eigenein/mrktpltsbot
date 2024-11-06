pub mod bot;
pub mod commands;
pub mod error;
pub mod methods;
pub mod objects;
pub mod render;
pub mod result;

use std::{fmt::Debug, time::Duration};

use backoff::{ExponentialBackoff, backoff::Backoff};
use futures::{Stream, StreamExt, TryStreamExt, stream};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use tokio::time::sleep;
use url::Url;

use crate::{
    client::Client,
    prelude::*,
    telegram::{
        error::TelegramError,
        methods::{AllowedUpdate, GetUpdates, Method},
        objects::Update,
        result::TelegramResult,
    },
};

/// Telegram bot API connection.
#[must_use]
#[derive(Clone)]
pub struct Telegram {
    client: Client,
    token: SecretString,
    root_url: Url,
}

impl Telegram {
    pub fn new(client: Client, token: SecretString) -> Result<Self> {
        Ok(Self {
            client,
            token,
            root_url: Url::parse("https://api.telegram.org")?,
        })
    }

    /// Call the Telegram Bot API method with automatic throttling and retrying.
    #[instrument(skip_all, fields(method = method.name()))]
    pub async fn call<M, R>(&self, method: &M) -> Result<R>
    where
        M: Method + ?Sized,
        R: Debug + DeserializeOwned,
    {
        let mut url = self.root_url.clone();
        url.set_path(&format!(
            "bot{}/{}",
            self.token.expose_secret(),
            method.name()
        ));

        let request_builder = self
            .client
            .request(reqwest::Method::POST, url)
            .json(method)
            .timeout(method.timeout());

        let mut backoff = ExponentialBackoff::default();
        loop {
            let result = request_builder
                .try_clone()?
                .read_json::<TelegramResult<R>>(false)
                .await;

            let error = match result {
                Ok(TelegramResult::Ok { result, .. }) => {
                    info!("Done");
                    break Ok(result);
                }

                Ok(TelegramResult::Err(TelegramError::TooManyRequests { retry_after, .. })) => {
                    warn!(retry_after.secs, "Throttling");
                    sleep(Duration::from_secs(retry_after.secs)).await;
                    continue;
                }

                Ok(TelegramResult::Err(error)) => anyhow!("Telegram Bot API error: {error:#}"),

                Err(error) => error,
            };

            if let Some(duration) = backoff.next_backoff() {
                warn!(?duration, "Retrying after the error: {error:#}",);
                sleep(duration).await;
            } else {
                warn!("All attempts have failed");
                break Err(error);
            }
        }
    }

    /// Convert the client into a [`Stream`] of Telegram [`Update`]'s.
    pub fn into_updates(
        self,
        offset: u64,
        poll_timeout_secs: u64,
    ) -> impl Stream<Item = Result<Update>> {
        let advance = move |(this, offset)| async move {
            let updates = GetUpdates::builder()
                .offset(offset)
                .timeout_secs(poll_timeout_secs)
                .allowed_updates(&[AllowedUpdate::Message])
                .build()
                .call_on(&this)
                .await?;
            let next_offset = updates
                .last()
                .map_or(offset, |last_update| last_update.id + 1);
            info!(n = updates.len(), next_offset, "Received Telegram updates");
            Ok::<_, Error>(Some((stream::iter(updates).map(Ok), (this, next_offset))))
        };
        stream::try_unfold((self, offset), advance).try_flatten()
    }
}
