use std::{collections::VecDeque, iter::once};

use maud::Render;

use crate::{
    marktplaats::listing::Listing,
    prelude::*,
    telegram::{
        methods::{InputMediaPhoto, Media, SendMediaGroup, SendMessage, SendPhoto},
        objects::{ChatId, LinkPreviewOptions, ParseMode},
        Telegram,
    },
};

/// Telegram request to send the corresponding listing.
pub enum SendListingRequest<'a> {
    Message(SendMessage<'a>),
    Photo(SendPhoto<'a>),
    MediaGroup(SendMediaGroup<'a>),
}

impl<'a> From<SendMessage<'a>> for SendListingRequest<'a> {
    fn from(send_message: SendMessage<'a>) -> Self {
        Self::Message(send_message)
    }
}

impl<'a> From<SendPhoto<'a>> for SendListingRequest<'a> {
    fn from(send_photo: SendPhoto<'a>) -> Self {
        Self::Photo(send_photo)
    }
}

impl<'a> From<SendMediaGroup<'a>> for SendListingRequest<'a> {
    fn from(send_media_group: SendMediaGroup<'a>) -> Self {
        Self::MediaGroup(send_media_group)
    }
}

impl<'a> SendListingRequest<'a> {
    pub fn with(chat_id: ChatId, listing: &'a Listing) -> Self {
        let html = listing.render().into_string();
        let mut image_urls: VecDeque<&str> = listing
            .pictures
            .iter()
            .filter_map(|picture| picture.any_url())
            .collect();

        match image_urls.len() {
            0 => SendMessage::builder()
                .chat_id(chat_id)
                .text(html)
                .parse_mode(ParseMode::Html)
                .link_preview_options(LinkPreviewOptions::builder().is_disabled(true).build())
                .build()
                .into(),

            1 => SendPhoto::builder()
                .chat_id(chat_id)
                .photo(image_urls[0])
                .caption(html)
                .parse_mode(ParseMode::Html)
                .build()
                .into(),

            _ => {
                let first_media = Media::InputMediaPhoto(
                    InputMediaPhoto::builder()
                        .media(image_urls.pop_front().unwrap())
                        .caption(html)
                        .parse_mode(ParseMode::Html)
                        .build(),
                );
                let other_media = image_urls
                    .into_iter()
                    .map(|url| InputMediaPhoto::builder().media(url).build())
                    .map(Media::InputMediaPhoto);
                let media = once(first_media).chain(other_media).collect();
                SendMediaGroup::builder()
                    .chat_id(chat_id)
                    .media(media)
                    .build()
                    .into()
            }
        }
    }

    pub async fn call_on(&self, telegram: &Telegram) -> Result {
        match self {
            Self::Message(request) => {
                let message = telegram.call(request).await?;
                debug!(message.id, "Sent");
            }
            Self::Photo(request) => {
                let message = telegram.call(request).await?;
                debug!(message.id, "Sent");
            }
            Self::MediaGroup(request) => {
                let messages = telegram.call(request).await?;
                for message in messages {
                    debug!(message.id, "Sent");
                }
            }
        };
        Ok(())
    }
}
