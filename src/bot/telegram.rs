use std::collections::HashSet;

use bon::Builder;
use futures::{Stream, stream};

use crate::{
    db::Db,
    marktplaats::Marktplaats,
    prelude::*,
    telegram::{methods::Method, objects::Update},
};

/// Telegram [`Message`] reactor.
#[derive(Builder)]
pub struct Reactor {
    authorized_chat_ids: HashSet<i64>,
    db: Db,
    marktplaats: Marktplaats,
}

impl Reactor {
    /// Run the reactor indefinitely and produce reactions.
    pub fn run(self, updates: impl Stream<Item = Result<Update>>) {
        todo!()
    }
}
