use reqwest_middleware::ClientWithMiddleware;
use url::Url;

use crate::prelude::*;

#[derive(Clone)]
pub struct Heartbeat(Option<HeartbeatInner>);

impl Heartbeat {
    pub fn new(client: ClientWithMiddleware, url: Option<Url>) -> Self {
        Self(url.map(|url| HeartbeatInner { client, url }))
    }

    pub async fn check_in(&self) {
        if let Err(error) = self.fallible_check_in().await {
            log::warn!("💔 Failed to send the heartbeat: {error:#}");
        }
    }

    async fn fallible_check_in(&self) -> Result {
        if let Some(inner) = &self.0 {
            inner.client.post(inner.url.clone()).send().await?.error_for_status()?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct HeartbeatInner {
    client: ClientWithMiddleware,
    url: Url,
}
