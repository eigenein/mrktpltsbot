pub mod bot;
pub mod commands;
pub mod methods;
pub mod objects;
pub mod reaction;
pub mod render;
pub mod result;

use std::fmt::Debug;

use reqwest_middleware::ClientWithMiddleware;
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use url::Url;

use crate::{
    prelude::*,
    telegram::{
        commands::CommandBuilder,
        methods::{GetMe, Method},
        result::TelegramResult,
    },
};

/// Telegram bot API connection.
#[must_use]
#[derive(Clone)]
pub struct Telegram {
    client: ClientWithMiddleware,
    token: SecretString,
    root_url: Url,
}

impl Telegram {
    pub fn new(client: ClientWithMiddleware, token: SecretString) -> Result<Self> {
        Ok(Self { client, token, root_url: Url::parse("https://api.telegram.org")? })
    }

    /// Call the Telegram Bot API method.
    #[instrument(skip_all)]
    pub async fn call<M, R>(&self, method: &M) -> Result<R>
    where
        M: Method + ?Sized,
        R: Debug + DeserializeOwned,
    {
        let mut url = self.root_url.clone();
        url.set_path(&format!("bot{}/{}", self.token.expose_secret(), method.name()));
        self.client
            .post(url)
            .json(method)
            .timeout(method.timeout())
            .send()
            .await?
            .json::<TelegramResult<R>>()
            .await?
            .into()
    }

    #[instrument(skip_all)]
    pub async fn command_builder(&self) -> Result<CommandBuilder> {
        let me = GetMe
            .call_on(self)
            .await
            .context("failed to get bot’s user")?
            .username
            .context("the bot has no username")?;
        CommandBuilder::new(&me)
    }
}
