use std::collections::HashSet;

use bon::Builder;

use crate::{db::Db, marktplaats::Marktplaats};

/// Telegram [`Message`] reactor.
#[derive(Builder)]
pub struct Reactor<M> {
    update_stream: M,
    authorized_chat_ids: HashSet<i64>,
    db: Db,
    marktplaats: Marktplaats,
}

impl<M> Reactor<M> {
    /// Run the [`Reactor`] indefinitely and react to [`Message`]'s.
    pub async fn run(self) {}
}
