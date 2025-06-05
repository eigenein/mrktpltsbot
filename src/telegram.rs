mod bot;
pub mod commands;
pub mod methods;
pub mod notification;
pub mod objects;
pub mod render;
mod response;

use std::{collections::BTreeMap, fmt::Debug};

use reqwest_middleware::ClientWithMiddleware;
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use url::Url;

pub use self::bot::Bot as TelegramBot;
use crate::{
    prelude::*,
    telegram::{
        commands::CommandBuilder,
        methods::{GetMe, Method},
        response::Response,
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
        let url = {
            let mut url = self.root_url.clone();
            url.set_path(&format!("bot{}/{}", self.token.expose_secret(), method.name()));
            url
        };
        let request_body = serde_json::to_value(method)?;
        let request = self.client.post(url).json(&request_body).timeout(method.timeout());
        sentry::configure_scope(|scope| {
            let context = BTreeMap::from([("request.body".to_string(), request_body)]);
            scope.set_context("telegram", sentry::protocol::Context::Other(context));
        });
        let response = request
            .send()
            .await
            .with_context(|| format!("failed to call `{}`", method.name()))?
            .json::<Response<R>>()
            .await?;
        Result::from(response).with_context(|| format!("`{}` failed", method.name()))
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
