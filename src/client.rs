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

/// [`reqwest::Client`] wrapper that encapsulates the client's settings.
#[derive(Clone)]
pub struct Client(reqwest::Client);

impl Client {
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

    const USER_AGENT: &'static str = concat!(
        "mrktpltsbot / ",
        crate_version!(),
        " (Rust; https://github.com/eigenein/mrktpltsbot)",
    );

    pub fn try_new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, HeaderValue::from_static(Self::USER_AGENT));
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

/// [`reqwest::RequestBuilder`] wrapper that traces the requests.
pub struct RequestBuilder(reqwest::RequestBuilder);

impl RequestBuilder {
    /// Send a JSON body.
    pub fn json<R: Serialize + ?Sized>(self, json: &R) -> Self {
        Self(self.0.json(json))
    }

    /// Override the request timeout.
    pub fn timeout(self, timeout: Duration) -> Self {
        Self(self.0.timeout(timeout))
    }

    #[instrument(skip_all, err(level = Level::DEBUG))]
    pub async fn read_json<R: DeserializeOwned>(self, error_for_status: bool) -> Result<R> {
        let body = self.read_text(error_for_status).await?;
        serde_json::from_str(&body).with_context(|| {
            format!("failed to deserialize the response into `{}`", type_name::<R>())
        })
    }

    #[instrument(skip_all, err(level = Level::TRACE))]
    pub async fn read_text(self, error_for_status: bool) -> Result<String> {
        let response = self.0.send().await.context("failed to send the request")?;
        let status = response.status();
        trace!(url = ?response.url(), ?status, "Reading response…");
        let body = response.text().await.context("failed to read the response")?;
        debug!(?status, body, "Received response");
        if error_for_status && (status.is_client_error() || status.is_server_error()) {
            Err(anyhow!("HTTP {status:?}"))
        } else {
            Ok(body)
        }
    }
}
