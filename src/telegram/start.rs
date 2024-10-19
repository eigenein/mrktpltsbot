use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::prelude::*;

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
pub struct StartCommand<'a> {
    pub username: &'a str,
    pub payload: StartPayload,
}

impl<'a> TryFrom<StartCommand<'a>> for Url {
    type Error = Error;

    /// Serialize the `/start` command into a [deep link][1].
    ///
    /// [1]: https://core.telegram.org/bots/features#deep-linking
    fn try_from(command: StartCommand<'a>) -> Result<Self, Self::Error> {
        let mut url = Url::parse("https://t.me")?;
        url.set_path(command.username);
        let payload = rmp_serde::to_vec_named(&command.payload)
            .context("failed to serialize the `/start` payload")?;
        url.set_query(Some(&format!("start={}", base64_url::encode(&payload))));
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_ok() -> Result {
        let payload = rmp_serde::to_vec_named(&StartPayload::Subscribe { query_hash: 1 })?;
        assert_eq!(payload, b"\x82\xA1t\xA3sub\xA1h\x01");
        assert_eq!(base64_url::encode(&payload), "gqF0o3N1YqFoAQ");
        Ok(())
    }

    #[test]
    fn test_into_url_ok() -> Result {
        let command = StartCommand {
            username: "mrktpltsbot",
            payload: StartPayload::Subscribe { query_hash: 1 },
        };
        let url = Url::try_from(command)?;
        assert_eq!(
            url.as_str(),
            "https://t.me/mrktpltsbot?start=gqF0o3N1YqFoAQ"
        );
        Ok(())
    }
}
