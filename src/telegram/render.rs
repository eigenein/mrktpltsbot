//! Listing rendering in Telegram.

use std::borrow::Cow;

use chrono_humanize::HumanTime;
use maud::{Markup, Render, html};
use url::Url;

use crate::marktplaats::listing::{
    Attribute,
    Condition,
    Delivery,
    Euro,
    Listing,
    Location,
    Price,
    Seller,
};

impl Render for Listing {
    fn render(&self) -> Markup {
        html! {
            strong { a href=(self.https_url()) { (self.title) } }
            "\n\n"
            (self.price)
            @for attribute in &self.attributes {
                (attribute)
            }
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
        let mut query = vec![("q", Cow::Borrowed(self.city_name.as_ref()))];
        if let (Some(latitude), Some(longitude)) = (self.latitude, self.longitude) {
            query.push(("ll", Cow::Owned(format!("{latitude},{longitude}"))));
        }
        html! {
            @match Url::parse_with_params("https://maps.apple.com/maps", &query) {
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

impl Render for Attribute {
    fn render(&self) -> Markup {
        html! {
            @match self {
                Self::Condition(condition) => { strong { " • " } (condition) },
                Self::Delivery(delivery) => { strong { " • " } (delivery) },
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
                Self::CollectionOrShipping => { (Self::CollectionOnly) strong { " • " } (Self::ShippingOnly) }
            }
        }
    }
}
