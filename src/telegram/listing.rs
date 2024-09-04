//! Listing rendering in Telegram.

use std::{collections::VecDeque, iter::once};

use chrono_humanize::HumanTime;
use maud::{html, Markup, Render};
use url::Url;

use crate::{
    marktplaats::listing::{Euro, Listing, Location, Price, Seller},
    telegram::{
        methods::{InputMediaPhoto, Media, SendMediaGroup, SendMessage, SendPhoto},
        objects::{ChatId, LinkPreviewOptions, ParseMode},
    },
};

impl Render for Listing {
    fn render(&self) -> Markup {
        html! {
            strong { a href=(self.https_url()) { (self.title) } }
            "\n\n"
            (self.price)
            "\n\n"
            blockquote expandable { (self.description()) }
            "\n\n"
            (self.seller)
            strong { " • " }
            (self.location)
            strong { " • " }
            (HumanTime::from(self.timestamp))
        }
    }
}

impl Render for Price {
    fn render(&self) -> Markup {
        html! {
            @match self {
                Self::Fixed { asking } => { strong { (Euro::from(*asking)) } }
                Self::OnRequest => { "❔ price on request" }
                Self::MinBid { asking } => { strong { (Euro::from(*asking)) } strong { " • " } "⬇️ bidding" }
                Self::SeeDescription => { }
                Self::ToBeAgreed => { "🤝 price to be agreed" }
                Self::Reserved => { "⚠️ reserved" }
                Self::FastBid => { "⬆️ bidding" }
                Self::Free => { em { "🆓 free" } }
                Self::Exchange => { "💱 exchange" }
            }
        }
    }
}

impl Render for Euro {
    fn render(&self) -> Markup {
        html! {
            "€" (self.0)
        }
    }
}

impl Render for Location {
    fn render(&self) -> Markup {
        let url = match (self.latitude, self.longitude) {
            (Some(latitude), Some(longitude)) => Url::parse_with_params(
                "https://maps.apple.com/maps",
                &[("ll", &format!("{latitude},{longitude}"))],
            ),
            _ => Url::parse_with_params("https://maps.apple.com/maps", &[("q", &self.city_name)]),
        };
        html! {
            @match url {
                Ok(url) => { a href=(url) { (self.city_name) } },
                Err(_) => (self.city_name)
            }
        }
    }
}

impl Render for Seller {
    fn render(&self) -> Markup {
        html! {
            a href=(format!("https://www.marktplaats.nl/u/{}/{}/", self.name, self.id)) {
                "@" (self.name)
            }
        }
    }
}

/// Telegram request to send the corresponding listing.
///
/// TODO: move somewhere else?
pub enum SendListingRequest<'a> {
    Message(SendMessage<'a>),
    Photo(SendPhoto<'a>),
    MediaGroup(SendMediaGroup<'a>),
}

impl<'a> SendListingRequest<'a> {
    pub fn build(chat_id: ChatId, listing: &'a Listing) -> Self {
        let html = listing.render().into_string();
        let mut image_urls: VecDeque<&str> = listing
            .pictures
            .iter()
            .filter_map(|picture| picture.any_url())
            .collect();
        if image_urls.is_empty() {
            let send_message = SendMessage::builder()
                .chat_id(chat_id)
                .text(html)
                .parse_mode(ParseMode::Html)
                .link_preview_options(LinkPreviewOptions::builder().is_disabled(true).build())
                .build();
            Self::Message(send_message)
        } else if image_urls.len() == 1 {
            let send_photo = SendPhoto::builder()
                .chat_id(chat_id)
                .photo(image_urls[0])
                .caption(html)
                .parse_mode(ParseMode::Html)
                .build();
            Self::Photo(send_photo)
        } else {
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
            let send_media_group = SendMediaGroup::builder()
                .chat_id(chat_id)
                .media(media)
                .build();
            Self::MediaGroup(send_media_group)
        }
    }
}
