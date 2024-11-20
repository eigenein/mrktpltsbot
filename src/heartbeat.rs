use reqwest::Method;
use url::Url;

use crate::{client::Client, prelude::*};

pub struct Heartbeat<'a>(Option<HeartbeatInner<'a>>);

impl<'a> Heartbeat<'a> {
    pub fn new(client: &'a Client, url: Option<Url>) -> Self {
        Self(url.map(|url| HeartbeatInner { client, url }))
    }

    pub async fn check_in(&self) {
        if let Some(inner) = &self.0 {
            if let Err(error) =
                inner.client.request(Method::POST, inner.url.clone()).read_text(true).await
            {
                warn!("Failed to send the heartbeat: {error:#}");
            }
        };
    }
}

struct HeartbeatInner<'a> {
    client: &'a Client,
    url: Url,
}
