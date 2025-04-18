use reqwest::Method;
use url::Url;

use crate::{client::Client, prelude::*};

#[derive(Clone)]
pub struct Heartbeat(Option<HeartbeatInner>);

impl Heartbeat {
    pub fn new(client: Client, url: Option<Url>) -> Self {
        Self(url.map(|url| HeartbeatInner { client, url }))
    }

    pub async fn check_in(&self) {
        if let Some(inner) = &self.0 {
            if let Err(error) =
                inner.client.request(Method::POST, inner.url.clone()).read_text(true).await
            {
                warn!("Failed to send the heartbeat: {error:#}");
            }
        }
    }
}

#[derive(Clone)]
struct HeartbeatInner {
    client: Client,
    url: Url,
}
