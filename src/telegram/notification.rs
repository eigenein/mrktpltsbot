use std::{collections::VecDeque, convert::TryFrom, iter::once};

use bon::bon;
use maud::{Render, html};
use url::Url;

use crate::{
    bot::query::SearchQuery,
    marktplaats::listing::Listing,
    prelude::*,
    telegram::{
        Telegram,
        methods::{InputMediaPhoto, Media, SendMediaGroup, SendMessage, SendPhoto},
        objects::{ChatId, LinkPreviewOptions, ParseMode, ReplyParameters},
        start::{StartCommand, StartPayload},
    },
};

pub enum Notification<'a> {
    Message(SendMessage<'a>),
    Photo(SendPhoto<'a>),
    MediaGroup(SendMediaGroup<'a>),
}

impl<'a> From<SendMessage<'a>> for Notification<'a> {
    fn from(send_message: SendMessage<'a>) -> Self {
        Self::Message(send_message)
    }
}

impl<'a> From<SendPhoto<'a>> for Notification<'a> {
    fn from(send_photo: SendPhoto<'a>) -> Self {
        Self::Photo(send_photo)
    }
}

impl<'a> From<SendMediaGroup<'a>> for Notification<'a> {
    fn from(send_media_group: SendMediaGroup<'a>) -> Self {
        Self::MediaGroup(send_media_group)
    }
}

#[bon]
impl<'a> Notification<'a> {
    #[builder]
    pub fn new(
        me: &'a str,
        query: SearchQuery<'a>,
        listing: &'a Listing,
        chat_id: ChatId,
        reply_parameters: Option<ReplyParameters>,
    ) -> Result<Self> {
        let command = StartCommand {
            username: me,
            payload: StartPayload::Subscribe {
                query_hash: query.hash,
            },
        };
        let caption = {
            let markup = html! {
                strong { a href=(listing.https_url()) { (listing.title) } }
                "\n"
                em { (query.text) }
                strong { " • " }
                a href=(Url::try_from(&command)?) { "Subscribe" }
                "\n\n"
                (listing.price)
                @for attribute in &listing.attributes {
                    (attribute)
                }
                "\n\n"
                blockquote expandable { (listing.description()) }
                "\n\n"
                (listing.seller)
                @if listing.location.city_name.is_some() {
                    strong { " • " }
                    (listing.location)
                }
            };
            markup.render().into_string()
        };

        let mut image_urls: VecDeque<&str> = listing
            .pictures
            .iter()
            .filter_map(|picture| picture.any_url())
            .collect();

        // Specific representation depends on how many pictures there are.
        let this = match image_urls.len() {
            0 => SendMessage::builder()
                .chat_id(chat_id)
                .text(caption)
                .parse_mode(ParseMode::Html)
                .link_preview_options(LinkPreviewOptions::builder().is_disabled(true).build())
                .maybe_reply_parameters(reply_parameters)
                .build()
                .into(),

            1 => SendPhoto::builder()
                .chat_id(chat_id)
                .photo(image_urls[0])
                .caption(caption)
                .parse_mode(ParseMode::Html)
                .maybe_reply_parameters(reply_parameters)
                .build()
                .into(),

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
                SendMediaGroup::builder()
                    .chat_id(chat_id)
                    .media(media)
                    .maybe_reply_parameters(reply_parameters)
                    .build()
                    .into()
            }
        };
        Ok(this)
    }

    pub async fn send_with(&self, telegram: &Telegram) -> Result {
        let messages = match self {
            Self::Message(request) => vec![telegram.call(request).await?],
            Self::Photo(request) => vec![telegram.call(request).await?],
            Self::MediaGroup(request) => telegram.call(request).await?,
        };
        for message in messages {
            debug!(message.id, "Sent");
        }
        Ok(())
    }
}
