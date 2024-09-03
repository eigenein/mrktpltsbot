use chrono_humanize::HumanTime;
use maud::{html, Markup, Render};

use crate::marktplaats::listing::{Euro, Listing, Price};

impl Render for Listing {
    fn render(&self) -> Markup {
        html! {
            strong { (self.title) }
            "\n"
            a href=(format!("https://www.marktplaats.nl/u/{}/{}/", self.seller.name, self.seller.id)) {
                "@" (self.seller.name)
            }
            " from "
            a href=(format!("https://maps.apple.com/maps?q={}", self.location.city_name)) {
                (self.location.city_name)
            }
            "\n"
            em { (HumanTime::from(self.timestamp)) }
            "\n\n"
            strong { "Price:" } " " (self.price)
            "\n\n"
            blockquote expandable { (self.description) }
        }
    }
}

impl Render for Price {
    fn render(&self) -> Markup {
        html! {
            @match self {
                Self::Fixed { asking } => { (Euro::from(*asking)) }
                Self::OnRequest => { "on request" }
                Self::MinBid { asking } => { (Euro::from(*asking)) " (bidding allowed)" }
                Self::SeeDescription => { "see description" }
                Self::ToBeAgreed => { "to be agreed" }
                Self::Reserved => { "reserved" }
                Self::FastBid => { "bid" }
                Self::Free => { "free" }
                Self::Exchange => { "exchange" }
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
