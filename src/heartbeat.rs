use reqwest::Method;
use url::Url;

use crate::{client::Client, prelude::*};

pub struct Heartbeat<'a>(Option<HeartbeatInner<'a>>);

impl<'a> Heartbeat<'a> {
    pub fn try_new(client: &'a Client, success_url: Option<Url>) -> Result<Self> {
        success_url
            .map(|success_url| HeartbeatInner::try_new(client, success_url))
            .transpose()
            .map(Self)
    }

    pub async fn report_success(&self) {
        let Some(inner) = &self.0 else {
            return;
        };
        let success_url = inner.success_url.clone();
        if let Err(error) = inner.client.request(Method::POST, success_url).read_text(true).await {
            warn!("Failed to send the heartbeat: {error:#}");
        }
    }

    pub async fn report_failure(&self, error: &Error) {
        let Some(inner) = &self.0 else {
            return;
        };
        let failure_url = inner.failure_url.clone();
        if let Err(error) = inner
            .client
            .request(Method::POST, failure_url)
            .body(error.to_string())
            .read_text(true)
            .await
        {
            warn!("Failed to report the failure: {error:#}");
        }
    }
}

struct HeartbeatInner<'a> {
    client: &'a Client,
    success_url: Url,
    failure_url: Url,
}

impl<'a> HeartbeatInner<'a> {
    fn try_new(client: &'a Client, success_url: Url) -> Result<Self> {
        let mut failure_url = success_url.clone();
        failure_url
            .path_segments_mut()
            .map_err(|_| anyhow!("could not add a segment to `{success_url}`"))?
            .push("fail");
        Ok(Self { client, success_url, failure_url })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_ok() -> Result {
        let client = Client::try_new()?;
        let url = Url::parse("https://uptime.betterstack.com/api/v1/heartbeat/XYZ1234")?;
        let heartbeat = Heartbeat::try_new(&client, Some(url))?;
        let inner = heartbeat.0.expect("inner should be `Some`");
        assert_eq!(
            inner.failure_url,
            Url::parse("https://uptime.betterstack.com/api/v1/heartbeat/XYZ1234/fail")?,
        );
        Ok(())
    }
}
