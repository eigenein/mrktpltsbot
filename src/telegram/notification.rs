use std::{collections::VecDeque, iter::once};

use bon::bon;
use serde::{Serialize, de::IgnoredAny};

use crate::{
    marktplaats::listing::Picture,
    telegram::{
        methods::{InputMediaPhoto, Media, Method, SendMediaGroup, SendMessage, SendPhoto},
        objects::{ChatId, LinkPreviewOptions, ParseMode, ReplyParameters},
    },
};

/// Rich HTML notification that call the correct method on-the-fly.
#[derive(Serialize)]
#[serde(untagged)]
pub enum SendNotification<'a> {
    /// Plain-text notification.
    Message(SendMessage<'a>),

    /// Single-photo notification.
    Photo(SendPhoto<'a>),

    /// Multiple-photo notification.
    MediaGroup(SendMediaGroup<'a>),
}

#[bon]
impl<'a> SendNotification<'a> {
    #[builder]
    pub fn new(
        caption: &'a str,
        pictures: &'a [Picture],
        chat_id: ChatId,
        reply_parameters: Option<ReplyParameters>,
    ) -> Self {
        let mut image_urls: VecDeque<&str> = pictures
            .iter()
            .filter_map(|picture| picture.any_url())
            .collect();

        // Specific representation depends on how many pictures there are.
        match image_urls.len() {
            0 => Self::Message(
                SendMessage::builder()
                    .chat_id(chat_id)
                    .text(caption)
                    .parse_mode(ParseMode::Html)
                    .link_preview_options(LinkPreviewOptions::builder().is_disabled(true).build())
                    .maybe_reply_parameters(reply_parameters)
                    .build(),
            ),

            1 => Self::Photo(
                SendPhoto::builder()
                    .chat_id(chat_id)
                    .photo(image_urls[0])
                    .caption(caption)
                    .parse_mode(ParseMode::Html)
                    .maybe_reply_parameters(reply_parameters)
                    .build(),
            ),

            _ => {
                let first_media = Media::InputMediaPhoto(
                    InputMediaPhoto::builder()
                        .media(image_urls.pop_front().unwrap())
                        .caption(caption)
                        .parse_mode(ParseMode::Html)
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

impl<'a> Method for SendNotification<'a> {
    type Response = IgnoredAny;

    fn name(&self) -> &'static str {
        match self {
            Self::Message(send_message) => send_message.name(),
            Self::Photo(send_photo) => send_photo.name(),
            Self::MediaGroup(send_media_group) => send_media_group.name(),
        }
    }
}
