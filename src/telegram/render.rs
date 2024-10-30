//! Listing rendering in Telegram.

use std::borrow::Cow;

use bon::{Builder, builder};
use maud::{Markup, PreEscaped, Render, html};
use url::Url;

use crate::{
    db::search_query::SearchQuery,
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
};

/// Just `<strong> • </strong>`.
pub const DELIMITER: PreEscaped<&'static str> = PreEscaped(
    // language=html
    "<strong> • </strong>",
);

#[derive(Builder)]
pub struct Link<M> {
    markup: M,
    url: Url,
}

impl<M: Render> Render for Link<M> {
    fn render(&self) -> Markup {
        html! { a href=(self.url) { (self.markup) } }
    }
}

#[derive(Builder)]
pub struct SimpleNotification<M> {
    markup: Markup,
    links: Vec<Link<M>>,
}

impl<M: Render> Render for SimpleNotification<M> {
    fn render(&self) -> Markup {
        html! {
            (self.markup)
            @for link in &self.links {
                (DELIMITER)
                (link)
            }
        }
    }
}

/// Render a simple message with links.
#[builder(finish_fn = render)]
pub fn simple_message<M: Render>(markup: M, links: &[Link<M>]) -> String {
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
#[builder(finish_fn = render)]
pub fn listing_description<M: Render>(
    listing: &Listing,
    search_query: &SearchQuery,
    links: &[Link<M>],
) -> String {
    let markup = html! {
        strong { a href=(listing.https_url()) { (listing.title) } }
        "\n"
        em { (search_query.text) }
        @for links in links {
            (DELIMITER)
            (links)
        }
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
            (DELIMITER)
            (listing.location)
        }
    };
    markup.render().into_string()
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
