//! `/start` command.

use bon::Builder;
use maud::{Markup, Render, html};
use prost::Message;

/// Start command with a [deep link][1].
///
/// [1]: https://core.telegram.org/bots/features#deep-linking
#[derive(Builder)]
pub struct StartCommand<'a> {
    pub me: &'a str,
    pub text: &'a str,
    pub payload: StartPayload,
}

impl<'a> Render for StartCommand<'a> {
    fn render(&self) -> Markup {
        let url = format!(
            "https://t.me/{}?start={}",
            self.me,
            base64_url::encode(&self.payload.encode_to_vec())
        );
        html! { a href=(url) { (self.text) } }
    }
}

/// Payload for a [`StartCommand`] with a [deep link][1].
///
/// [1]: https://core.telegram.org/bots/features#deep-linking
#[derive(Message)]
pub struct StartPayload {
    #[prost(tag = "1", message, optional)]
    pub subscribe: Option<SubscriptionStartCommand>,

    #[prost(tag = "2", message, optional)]
    pub unsubscribe: Option<SubscriptionStartCommand>,
}

impl StartPayload {
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
    fn test_render_start_command_ok() {
        let command = StartCommand::builder()
            .me("mrktpltsbot")
            .payload(StartPayload::subscribe_to(42))
            .text("Subscribe")
            .build();

        // language=html
        assert_eq!(
            command.render().into_string(),
            r#"<a href="https://t.me/mrktpltsbot?start=CgkJKgAAAAAAAAA">Subscribe</a>"#,
        );
    }
}
