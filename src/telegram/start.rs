use bon::Builder;
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
