//! `/start` command.

use bon::{Builder, bon};
use maud::Render;
use prost::Message;
use url::Url;

use crate::{db::query_hash::QueryHash, prelude::*, telegram::render::Link};

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

    /// Build a new command link.
    #[builder(finish_fn = build)]
    pub fn link<C: Render>(&self, content: C, payload: &CommandPayload) -> Link<C> {
        let mut url = self.0.clone();
        url.query_pairs_mut()
            .append_pair("start", &payload.to_base64());
        Link::builder().content(content).url(url).build()
    }
}

/// Payload for a `/start` command with a [deep link][1].
///
/// [1]: https://core.telegram.org/bots/features#deep-linking
#[derive(Builder, Message)]
pub struct CommandPayload {
    #[prost(tag = "1", message, optional)]
    pub subscribe: Option<SubscriptionStartCommand>,

    #[prost(tag = "2", message, optional)]
    pub unsubscribe: Option<SubscriptionStartCommand>,
}

impl CommandPayload {
    pub fn from_base64(text: &str) -> Result<Self> {
        let payload = base64_url::decode(text).context("failed to decode the payload")?;
        Self::decode(payload.as_slice()).context("failed to deserialize the payload")
    }

    pub fn to_base64(&self) -> String {
        base64_url::encode(&self.encode_to_vec())
    }
}

#[derive(Eq, PartialEq, Message)]
pub struct SubscriptionStartCommand {
    #[prost(tag = "1", fixed64)]
    pub query_hash: u64,
}

impl SubscriptionStartCommand {
    pub const fn new(query_hash: QueryHash) -> Self {
        Self {
            query_hash: query_hash.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_subscribe_link_ok() -> Result {
        let builder = CommandBuilder::new("mrktpltsbot")?;
        let command = CommandPayload::builder()
            .subscribe(SubscriptionStartCommand::new(QueryHash(
                17_108_638_805_232_950_527,
            )))
            .build();
        let link = builder
            .link()
            .content("Subscribe")
            .payload(&command)
            .build();

        // language=html
        assert_eq!(
            link.render().into_string(),
            r#"<a href="https://t.me/mrktpltsbot?start=CgkJ_5xfEFkYbu0">Subscribe</a>"#,
        );

        Ok(())
    }

    #[test]
    fn test_deserialize_payload_ok() -> Result {
        let payload = CommandPayload::from_base64("CgkJ_5xfEFkYbu0")?;
        assert_eq!(
            payload.subscribe,
            Some(SubscriptionStartCommand {
                query_hash: 17_108_638_805_232_950_527
            })
        );
        Ok(())
    }
}
