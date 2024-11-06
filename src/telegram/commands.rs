//! `/start` command.

use bon::{Builder, bon};
use maud::Render;
use prost::{Enumeration, Message};
use url::Url;

use crate::{prelude::*, telegram::render::Link};

/// Builder of `/start` commands with [deep linking][1].
///
/// [1]: https://core.telegram.org/bots/features#deep-linking
#[derive(Clone)]
#[must_use]
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
    #[prost(tag = "3", message, optional)]
    pub subscription: Option<SubscriptionCommand>,
}

impl CommandPayload {
    pub fn from_base64(text: &str) -> Result<Self> {
        let payload = base64_url::decode(text).context("failed to decode the payload")?;
        Self::decode(payload.as_slice()).context("failed to deserialize the payload")
    }

    pub fn to_base64(&self) -> String {
        base64_url::encode(&self.encode_to_vec())
    }

    pub const fn subscribe_to(query_hash: i64) -> Self {
        Self {
            subscription: Some(SubscriptionCommand::subscribe_to(query_hash)),
        }
    }

    pub const fn unsubscribe_from(query_hash: i64) -> Self {
        Self {
            subscription: Some(SubscriptionCommand::unsubscribe_from(query_hash)),
        }
    }
}

#[derive(Eq, PartialEq, Message)]
pub struct SubscriptionCommand {
    #[prost(tag = "1", sfixed64)]
    pub query_hash: i64,

    #[prost(tag = "2", enumeration = "SubscriptionAction")]
    pub action: i32,
}

impl SubscriptionCommand {
    pub const fn subscribe_to(query_hash: i64) -> Self {
        Self {
            query_hash,
            action: SubscriptionAction::Subscribe as i32,
        }
    }

    pub const fn unsubscribe_from(query_hash: i64) -> Self {
        Self {
            query_hash,
            action: SubscriptionAction::Unsubscribe as i32,
        }
    }
}

#[derive(Debug, Enumeration)]
#[repr(i32)]
pub enum SubscriptionAction {
    None = 0,
    Subscribe = 1,
    Unsubscribe = 2,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::search_query::SearchQuery;

    #[test]
    fn test_build_subscribe_link_ok() -> Result {
        let search_query = SearchQuery::from("unifi".to_string());
        let command = CommandPayload::builder()
            .subscription(SubscriptionCommand::subscribe_to(search_query.hash))
            .build();
        let link = CommandBuilder::new("mrktpltsbot")?
            .link()
            .content("Subscribe")
            .payload(&command)
            .build();

        // language=html
        assert_eq!(
            link.render().into_string(),
            r#"<a href="https://t.me/mrktpltsbot?start=GgsJ_5xfEFkYbu0QAQ">Subscribe</a>"#,
        );

        Ok(())
    }

    #[test]
    fn test_deserialize_payload_ok() -> Result {
        let payload = CommandPayload::from_base64("GgsJ_5xfEFkYbu0QAQ")?;
        assert_eq!(
            payload.subscription,
            Some(SubscriptionCommand::subscribe_to(
                -1_338_105_268_476_601_089
            ))
        );
        Ok(())
    }
}
