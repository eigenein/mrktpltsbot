//! Listing rendering in Telegram.

use std::borrow::Cow;

use bon::Builder;
use maud::{Markup, PreEscaped, Render, html};
use url::Url;

use crate::{
    marktplaats::listing::{
        Attribute,
        Condition,
        Delivery,
        Euro,
        Listing,
        Location,
        Price,
        Seller,
    },
    telegram::objects::ChatId,
};

/// Just `<strong> • </strong>`.
pub const DELIMITER: PreEscaped<&'static str> = PreEscaped(
    // language=html
    "<strong> • </strong>",
);

pub fn unauthorized(chat_id: &ChatId) -> Markup {
    html! {
        "👋 Thank you for your interest"
        "\n\n"
        "This bot cannot handle many users, so it is private and only intended for authorized users."
        "\n\n"
        "However, " strong { "its " a href="https://github.com/eigenein/mrktpltsbot" { "source code" } " is open" } ","
        " and you are free to deploy your own instance."
        "\n\n"
        "If you are already setting it up for yourself, or someone is setting it up for you,"
        " "
        strong { "the following ID should be added to the list of authorized chat IDs:" }
        "\n\n"
        pre { code { (chat_id) } }
    }
}

/// Render a simple message with links.
pub fn simple_message<M1: Render, M2: Render>(markup: M1, links: &[Link<M2>]) -> String {
    let markup = html! {
        (markup)
        @for links in links {
            (DELIMITER)
            (links)
        }
    };
    markup.render().into_string()
}

/// Render the listing description.
pub fn listing_description<M: Render>(
    listing: &Listing,
    manage_search_query: &ManageSearchQuery<'_, M>,
) -> String {
    let markup = html! {
        strong { a href=(listing.https_url()) { (listing.title) } }
        "\n"
        (manage_search_query)
        "\n\n"
        (listing.price)
        @for attribute in &listing.attributes {
            (attribute)
        }
        "\n\n"
        blockquote { (listing.description()) }
        "\n\n"
        (listing.seller)
        @if listing.location.city_name.is_some() {
            (DELIMITER)
            (listing.location)
        }
    };
    markup.render().into_string()
}

#[derive(Builder)]
pub struct Link<C> {
    content: C,
    url: Url,
}

impl<C: Render> Render for Link<C> {
    fn render(&self) -> Markup {
        html! { a href=(self.url) { (self.content) } }
    }
}

impl Render for ChatId {
    fn render(&self) -> Markup {
        html! {
            @match self {
                Self::Integer(chat_id) => code { (chat_id) },
                Self::Username(username) => code { (username) },
            }
        }
    }
}

impl Render for Price {
    fn render(&self) -> Markup {
        html! {
            @match self {
                Self::Fixed { asking } => { strong { (Euro::from(*asking)) } }
                Self::OnRequest => { "❔ price on request" }
                Self::MinBid { asking } => { strong { (Euro::from(*asking)) } (DELIMITER) "⬇️ bidding" }
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
        let Some(city_name) = self.city_name.as_deref() else {
            return Markup::default();
        };
        let mut query = vec![("q", Cow::Borrowed(city_name))];
        if let (Some(latitude), Some(longitude)) = (self.latitude, self.longitude) {
            query.push(("ll", Cow::Owned(format!("{latitude},{longitude}"))));
        }
        html! {
            @match Url::parse_with_params("https://maps.apple.com/maps", &query) {
                Ok(url) => { a href=(url) { (city_name) } },
                Err(_) => (city_name)
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

impl Render for Attribute {
    fn render(&self) -> Markup {
        html! {
            @match self {
                Self::Condition(condition) => { (DELIMITER) (condition) },
                Self::Delivery(delivery) => { (DELIMITER) (delivery) },
                Self::Other(_) => {},
            }
        }
    }
}

impl Render for Condition {
    fn render(&self) -> Markup {
        html! {
            @match self {
                Self::New => "🟢 new",
                Self::AsGoodAsNew => "🟡 as good as new",
                Self::Refurbished => "🟡 refurbished",
                Self::Used => "🟠 used",
                Self::NotWorking => "⛔️ not working",
            }
        }
    }
}

impl Render for Delivery {
    fn render(&self) -> Markup {
        html! {
            @match self {
                Self::CollectionOnly => "🚶 collection",
                Self::ShippingOnly => "📦 shipping",
                Self::CollectionOrShipping => { (Self::CollectionOnly) (DELIMITER) (Self::ShippingOnly) }
            }
        }
    }
}

/// Search query as a text together with the management links.
#[derive(Copy, Clone)]
pub struct ManageSearchQuery<'a, C> {
    search_query: &'a str,
    links: &'a [&'a Link<C>],
}

impl<'a, C> ManageSearchQuery<'a, C> {
    pub const fn new(search_query: &'a str, links: &'a [&'a Link<C>]) -> Self {
        Self { search_query, links }
    }
}

impl<'a, C: Render> Render for ManageSearchQuery<'a, C> {
    fn render(&self) -> Markup {
        html! {
            em { (self.search_query) }
            @for links in self.links {
                (DELIMITER) (links)
            }
        }
    }
}
