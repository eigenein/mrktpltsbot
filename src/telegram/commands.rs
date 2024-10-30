//! `/start` command.

use bon::bon;
use maud::Render;
use prost::Message;
use url::Url;

use crate::{prelude::*, telegram::render::Link};

/// Builder of `/start` commands with [deep linking][1].
///
/// [1]: https://core.telegram.org/bots/features#deep-linking
pub struct CommandBuilder(Url);

#[bon]
impl CommandBuilder {
    pub fn new(me: &str) -> Result<Self> {
        let mut base_url = Url::parse("https://t.me/")?;
        base_url.set_path(me);
        Ok(Self(base_url))
    }

    /// Build a new command.
    #[builder(finish_fn = build)]
    pub fn command<M: Render>(&self, markup: M, payload: &CommandPayload) -> Link<M> {
        let mut url = self.0.clone();
        url.query_pairs_mut()
            .append_pair("start", &base64_url::encode(&payload.encode_to_vec()));
        Link::builder().markup(markup).url(url).build()
    }
}

/// Payload for a `/start` command with a [deep link][1].
///
/// [1]: https://core.telegram.org/bots/features#deep-linking
#[derive(Message)]
pub struct CommandPayload {
    #[prost(tag = "1", message, optional)]
    pub subscribe: Option<SubscriptionStartCommand>,

    #[prost(tag = "2", message, optional)]
    pub unsubscribe: Option<SubscriptionStartCommand>,
}

impl CommandPayload {
    pub const fn subscribe_to(query_hash: u64) -> Self {
        Self {
            subscribe: Some(SubscriptionStartCommand { query_hash }),
            unsubscribe: None,
        }
    }

    pub const fn unsubscribe_from(query_hash: u64) -> Self {
        Self {
            subscribe: None,
            unsubscribe: Some(SubscriptionStartCommand { query_hash }),
        }
    }
}

#[derive(Message)]
pub struct SubscriptionStartCommand {
    #[prost(tag = "1", fixed64)]
    pub query_hash: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_command_ok() -> Result {
        let builder = CommandBuilder::new("mrktpltsbot")?;
        let link = builder
            .command()
            .markup("Subscribe")
            .payload(&CommandPayload::subscribe_to(42))
            .build();

        // language=html
        assert_eq!(
            link.render().into_string(),
            r#"<a href="https://t.me/mrktpltsbot?start=CgkJKgAAAAAAAAA">Subscribe</a>"#,
        );

        Ok(())
    }
}
