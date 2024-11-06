use bon::Builder;
use futures::{Stream, stream};

use crate::{db::Db, marktplaats::Marktplaats, prelude::*, telegram::methods::AnyMethod};

/// Marktplaats reactor.
#[derive(Builder)]
pub struct Reactor<'s> {
    db: &'s Db,
    marktplaats: &'s Marktplaats,
}

impl<'s> Reactor<'s> {
    /// Run the reactor indefinitely and produce reactions.
    pub fn run(&'s self) -> impl Stream<Item = Result<AnyMethod<'static>>> + 's {
        stream::empty() // TODO
    }
}
