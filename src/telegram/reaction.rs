use std::{borrow::Cow, collections::VecDeque, iter::once};

use bon::bon;
use serde::{Serialize, de::IgnoredAny};

use crate::{
    marktplaats::listing::Picture,
    telegram::{
        methods::{InputMediaPhoto, Media, Method, SendMediaGroup, SendMessage, SendPhoto},
        objects::{ChatId, LinkPreviewOptions, ParseMode, ReplyParameters},
    },
};

#[derive(Serialize)]
#[serde(untagged)]
pub enum Reaction<'a> {
    MediaGroup(SendMediaGroup<'a>),
    Message(SendMessage<'a>),
    Photo(SendPhoto<'a>),
}

impl<'a> From<SendMessage<'a>> for Reaction<'a> {
    fn from(value: SendMessage<'a>) -> Self {
        Self::Message(value)
    }
}

#[bon]
impl<'a> Reaction<'a> {
    /// Build a new method from a listing contents.
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

impl<'a> Method for Reaction<'a> {
    type Response = IgnoredAny;

    fn name(&self) -> &'static str {
        match self {
            Self::MediaGroup(send_media_group) => send_media_group.name(),
            Self::Message(send_message) => send_message.name(),
            Self::Photo(send_photo) => send_photo.name(),
        }
    }
}
