//! Listing rendering in Telegram.

use std::borrow::Cow;

use bon::Builder;
use maud::{Markup, Render, html};
use url::Url;

use crate::{
    bot::query::SearchQuery,
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
    prelude::*,
    telegram::start::StartCommand,
};

pub trait TryRender {
    fn try_render(&self) -> Result<Markup>;
}

impl<R: Render> TryRender for R {
    fn try_render(&self) -> Result<Markup> {
        Ok(self.render())
    }
}

#[derive(Builder)]
pub struct ListingCaption<'a> {
    search_query: SearchQuery,
    listing: &'a Listing,
    commands: &'a [StartCommand<'a>],
}

impl<'a> TryRender for ListingCaption<'a> {
    fn try_render(&self) -> Result<Markup> {
        Ok(html! {
            strong { a href=(self.listing.https_url()) { (self.listing.title) } }
            "\n"
            em { (self.search_query.text) }
            @for command in self.commands {
                strong { " • " }
                (command.try_render()?)
            }
            "\n\n"
            (self.listing.price)
            @for attribute in &self.listing.attributes {
                (attribute)
            }
            "\n\n"
            blockquote expandable { (self.listing.description()) }
            "\n\n"
            (self.listing.seller)
            @if self.listing.location.city_name.is_some() {
                strong { " • " }
                (self.listing.location)
            }
        })
    }
}

impl<'a> TryRender for StartCommand<'a> {
    fn try_render(&self) -> Result<Markup> {
        let mut url = Url::parse("https://t.me")?;
        url.set_path(self.me);
        let payload = rmp_serde::to_vec_named(&self.payload)
            .context("failed to serialize the `/start` payload")?;
        url.set_query(Some(&format!("start={}", base64_url::encode(&payload))));
        Ok(html! { a href=(url) { (self.text) } })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telegram::start::StartPayload;

    #[test]
    fn test_render_start_command_ok() -> Result {
        let command = StartCommand::builder()
            .me("mrktpltsbot")
            .payload(StartPayload::Subscribe { query_hash: 1 })
            .text("Subscribe")
            .build();
        assert_eq!(
            command.try_render()?.into_string(),
            r#"<a href="https://t.me/mrktpltsbot?start=gqF0o3N1YqFoAQ">Subscribe</a>"#,
        );
        Ok(())
    }
}
