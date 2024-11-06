use std::{borrow::Cow, collections::VecDeque, iter::once};

use bon::bon;
use serde::Serialize;

use crate::{
    marktplaats::listing::Picture,
    prelude::*,
    telegram::{
        Telegram,
        methods::{InputMediaPhoto, Media, Method, SendMediaGroup, SendMessage, SendPhoto},
        objects::{ChatId, LinkPreviewOptions, ParseMode, ReplyParameters},
    },
};

/// Well… a reaction to something.
pub struct Reaction<'a> {
    /// Methods to execute on Telegram to produce the reaction.
    pub methods: Vec<ReactionMethod<'a>>,
}

impl<'a> Reaction<'a> {
    /// Send the reaction to the specified [`Telegram`] connection.
    pub async fn react_to(&self, telegram: &Telegram) -> Result {
        for method in &self.methods {
            match method {
                ReactionMethod::Message(inner) => inner.call_discarded_on(telegram).await?,
                ReactionMethod::Photo(inner) => inner.call_discarded_on(telegram).await?,
                ReactionMethod::MediaGroup(inner) => inner.call_discarded_on(telegram).await?,
            }
        }
        Ok(())
    }
}

impl<'a> From<ReactionMethod<'a>> for Reaction<'a> {
    fn from(method: ReactionMethod<'a>) -> Self {
        Self {
            methods: vec![method],
        }
    }
}

impl<'a> From<Vec<SendMessage<'a>>> for Reaction<'a> {
    fn from(send_messages: Vec<SendMessage<'a>>) -> Self {
        Self {
            methods: send_messages
                .into_iter()
                .map(ReactionMethod::Message)
                .collect(),
        }
    }
}

impl<'a> From<SendMessage<'a>> for Reaction<'a> {
    fn from(send_message: SendMessage<'a>) -> Self {
        ReactionMethod::Message(send_message).into()
    }
}

/// Reaction method on Telegram.
#[derive(Serialize)]
#[serde(untagged)]
pub enum ReactionMethod<'a> {
    MediaGroup(SendMediaGroup<'a>),
    Message(SendMessage<'a>),
    Photo(SendPhoto<'a>),
}

#[bon]
impl<'a> ReactionMethod<'a> {
    /// Build a new reaction method from a listing contents.
    #[builder(finish_fn = build)]
    pub fn from_listing(
        chat_id: Cow<'a, ChatId>,
        #[builder(into)] text: Cow<'a, str>,
        parse_mode: ParseMode,
        #[builder(into)] pictures: Vec<Picture>,
        reply_parameters: Option<ReplyParameters>,
    ) -> Self {
        let mut image_urls: VecDeque<_> =
            pictures.into_iter().filter_map(Picture::into_url).collect();

        // Specific representation depends on how many pictures there are.
        match image_urls.len() {
            0 => Self::Message(
                SendMessage::builder()
                    .chat_id(chat_id)
                    .text(text)
                    .parse_mode(parse_mode)
                    .link_preview_options(LinkPreviewOptions::DISABLED)
                    .maybe_reply_parameters(reply_parameters)
                    .build(),
            ),

            1 => Self::Photo(
                // We cannot send one photo as a «media group», so sending it as a «photo».
                SendPhoto::builder()
                    .chat_id(chat_id)
                    .photo(image_urls.pop_front().unwrap())
                    .caption(text)
                    .parse_mode(parse_mode)
                    .maybe_reply_parameters(reply_parameters)
                    .build(),
            ),

            _ => {
                let first_media = Media::InputMediaPhoto(
                    // Telegram needs the description in the first photo's caption.
                    InputMediaPhoto::builder()
                        .media(image_urls.pop_front().unwrap())
                        .caption(text)
                        .parse_mode(parse_mode)
                        .build(),
                );
                let other_media = image_urls
                    .into_iter()
                    .map(|url| InputMediaPhoto::builder().media(url).build())
                    .map(Media::InputMediaPhoto);
                let media = once(first_media).chain(other_media).collect();
                Self::MediaGroup(
                    SendMediaGroup::builder()
                        .chat_id(chat_id)
                        .media(media)
                        .maybe_reply_parameters(reply_parameters)
                        .build(),
                )
            }
        }
    }
}
