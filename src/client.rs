//! Provides the global `Client` instance.

use std::time::Duration;

use clap::crate_version;
use reqwest::{
    header,
    header::{HeaderMap, HeaderValue},
};
use reqwest_middleware::ClientWithMiddleware;

use crate::prelude::*;

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const USER_AGENT: &str = concat!(
    "mrktpltsbot / ",
    crate_version!(),
    " (Rust; https://github.com/eigenein/mrktpltsbot)",
);

pub fn try_new(connection_verbose: bool) -> Result<ClientWithMiddleware> {
    let _span =
        span!("🌏 Instantiating client…", connection_verbose = connection_verbose).entered();
    let mut headers = HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static(USER_AGENT));
    let client = reqwest::Client::builder()
        .gzip(true)
        .use_rustls_tls()
        .default_headers(headers)
        .timeout(DEFAULT_TIMEOUT)
        .connect_timeout(DEFAULT_TIMEOUT)
        .pool_idle_timeout(Some(Duration::from_secs(300)))
        .connection_verbose(connection_verbose)
        .build()
        .context("failed to build an HTTP client")?;
    let client = reqwest_middleware::ClientBuilder::new(client).build();
    Ok(client)
}
