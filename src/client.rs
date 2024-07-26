//! Provides the global `Client` instance.

use std::time::Duration;

use clap::crate_version;
use reqwest::{
    header,
    header::{HeaderMap, HeaderValue},
    Client,
};

use crate::prelude::*;

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

pub fn build_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        HeaderValue::from_static(concat!(
            "mrktpltsbot / ",
            crate_version!(),
            " (Rust; https://github.com/eigenein/mrktpltsbot)",
        )),
    );
    Client::builder()
        .gzip(true)
        .use_rustls_tls()
        .default_headers(headers)
        .timeout(DEFAULT_TIMEOUT)
        .pool_idle_timeout(Some(Duration::from_secs(600)))
        .build()
        .context("failed to build an HTTP client")
}
