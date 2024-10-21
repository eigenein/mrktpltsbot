//! Provides the global `Client` instance.

use std::{any::type_name, time::Duration};

use clap::crate_version;
use reqwest::{
    IntoUrl,
    Method,
    header,
    header::{HeaderMap, HeaderValue},
};
use serde::{Serialize, de::DeserializeOwned};

use crate::prelude::*;

#[derive(Clone)]
pub struct Client(reqwest::Client);

impl Client {
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

    const USER_AGENT: &'static str = concat!(
        "mrktpltsbot / ",
        crate_version!(),
        " (Rust; https://github.com/eigenein/mrktpltsbot)",
    );

    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static(Self::USER_AGENT),
        );
        reqwest::Client::builder()
            .gzip(true)
            .use_rustls_tls()
            .default_headers(headers)
            .timeout(Self::DEFAULT_TIMEOUT)
            .connect_timeout(Self::DEFAULT_TIMEOUT)
            .pool_idle_timeout(Some(Duration::from_secs(300)))
            .build()
            .context("failed to build an HTTP client")
            .map(Self)
    }

    pub fn request(&self, method: Method, url: impl IntoUrl) -> RequestBuilder {
        RequestBuilder(self.0.request(method, url))
    }
}

pub struct RequestBuilder(reqwest::RequestBuilder);

impl RequestBuilder {
    pub fn try_clone(&self) -> Result<Self> {
        self.0
            .try_clone()
            .context("failed to clone the request builder")
            .map(Self)
    }

    pub fn json<R: Serialize + ?Sized>(self, json: &R) -> Self {
        Self(self.0.json(json))
    }

    pub fn timeout(self, timeout: Duration) -> Self {
        Self(self.0.timeout(timeout))
    }

    #[instrument(skip_all, ret(level = Level::DEBUG), err(level = Level::DEBUG))]
    pub async fn read_text(self, error_for_status: bool) -> Result<String> {
        let response = self.0.send().await.context("failed to send the request")?;
        let status = response.status();
        let body = response
            .text()
            .await
            .context("failed to read the response")?;
        trace!(?status, body, "Received response");
        if error_for_status && (status.is_client_error() || status.is_server_error()) {
            Err(anyhow!("HTTP {status:?}"))
        } else {
            Ok(body)
        }
    }

    pub async fn read_json<R: DeserializeOwned>(self, error_for_status: bool) -> Result<R> {
        let body = self.read_text(error_for_status).await?;
        serde_json::from_str(&body).with_context(|| {
            format!(
                "failed to deserialize the response into `{}`",
                type_name::<R>()
            )
        })
    }
}
