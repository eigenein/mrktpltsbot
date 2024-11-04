use bon::builder;
use futures::{Stream, StreamExt, TryStreamExt, stream};

use crate::{
    prelude::*,
    telegram::{
        Telegram,
        methods::{AllowedUpdate, GetUpdates, Method},
        objects::Update,
    },
};

/// [`Stream`] of Telegram [`Update`]'s.
#[builder(finish_fn = build)]
pub fn update_stream(
    telegram: Telegram,
    offset: u64,
    poll_timeout_secs: u64,
) -> impl Stream<Item = Result<Update>> {
    stream::try_unfold((telegram, offset), move |(telegram, offset)| async move {
        let updates = GetUpdates::builder()
            .offset(offset)
            .timeout_secs(poll_timeout_secs)
            .allowed_updates(&[AllowedUpdate::Message])
            .build()
            .call_on(&telegram)
            .await?;
        let next_offset = updates
            .last()
            .map_or(offset, |last_update| last_update.id + 1);
        info!(n = updates.len(), next_offset, "Received Telegram updates");
        Ok::<_, Error>(Some((
            stream::iter(updates).map(Ok),
            (telegram, next_offset),
        )))
    })
    .try_flatten()
}
