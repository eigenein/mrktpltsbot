use bon::Builder;
use serde::{Deserialize, Serialize};

use crate::{bot::query::SearchQuery, prelude::*};

/// [Deep link][1] payload.
///
/// [1]: https://core.telegram.org/bots/features#deep-linking
#[derive(Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum StartPayload {
    #[serde(rename = "sub")]
    Subscribe {
        #[serde(rename = "h")]
        query_hash: i64,
    },

    #[serde(rename = "unsub")]
    Unsubscribe {
        #[serde(rename = "h")]
        query_hash: i64,
    },
}

/// Start command with a [deep link][1], for example: `https://t.me/mrktpltsbot?start=gqF0o3N1YqFoAQ`.
///
/// [1]: https://core.telegram.org/bots/features#deep-linking
#[derive(Builder)]
pub struct StartCommand<'a> {
    pub me: &'a str,
    pub text: &'a str,
    pub payload: StartPayload,
}

impl SearchQuery {
    pub const fn subscribe(&self) -> StartPayload {
        StartPayload::Subscribe {
            query_hash: self.hash,
        }
    }

    pub const fn unsubscribe(&self) -> StartPayload {
        StartPayload::Unsubscribe {
            query_hash: self.hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_ok() -> Result {
        let payload = rmp_serde::to_vec_named(&StartPayload::Subscribe { query_hash: 1 })?;
        assert_eq!(payload, b"\x82\xA1t\xA3sub\xA1h\x01");
        Ok(())
    }
}