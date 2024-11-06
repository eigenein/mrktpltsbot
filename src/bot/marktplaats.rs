use bon::Builder;
use futures::{Stream, stream};

use crate::{db::Db, marktplaats::Marktplaats, telegram::methods::AnyMethod};

/// Marktplaats reactor.
#[derive(Builder)]
pub struct Reactor {
    db: Db,
    marktplaats: Marktplaats,
}

impl Reactor {
    /// Run the reactor indefinitely and produce reactions.
    pub fn run<'s>(
        &'s self,
    ) -> impl Stream<Item = crate::prelude::Result<Vec<AnyMethod<'static>>>> + 's {
        stream::empty() // TODO
    }
}
