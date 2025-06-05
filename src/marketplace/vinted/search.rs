use bon::Builder;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    marketplace::{
        item::Amount,
        vinted::{VintedError, client::VintedClient},
    },
    prelude::*,
};

#[must_use]
#[derive(Builder, Serialize)]
pub struct SearchRequest<'a> {
    #[serde(skip)]
    pub access_token: &'a str,

    #[builder(default = 1)]
    pub page: u32,

    pub per_page: u32,

    pub search_text: &'a str,

    #[builder(default = Order::NewestFirst)]
    pub order: Order,
}

impl SearchRequest<'_> {
    pub async fn call_on(&self, client: &VintedClient) -> Result<SearchResults, VintedError> {
        client.search(self.access_token, self).await
    }
}

#[must_use]
#[derive(Serialize)]
pub enum Order {
    #[serde(rename = "newest_first")]
    NewestFirst,
}

#[derive(Debug, Deserialize)]
pub struct SearchResults {
    pub items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub id: i64,
    pub title: String,
    pub brand_title: String,
    pub price: Price,
    pub url: Url,
    pub photo: Photo,
    pub user: User,
    pub status: Status,
}

impl From<Item> for crate::marketplace::item::Item {
    fn from(item: Item) -> Self {
        Self::builder()
            .id(format!("vinted::{}", item.id))
            .url(item.url)
            .title(item.title)
            .picture_url(item.photo.full_size_url)
            .condition(item.status.into())
            .delivery(crate::marketplace::item::Delivery::ShippingOnly)
            .price(item.price.into())
            .seller(item.user.into())
            .maybe_location(None)
            .build()
    }
}

#[derive(Debug, Deserialize)]
pub struct Price {
    #[serde(deserialize_with = "Amount::deserialize_from_string")]
    pub amount: Amount,
}

impl From<Price> for crate::marketplace::item::Price {
    fn from(price: Price) -> Self {
        Self::MaximalBid(price.amount)
    }
}

#[derive(Debug, Deserialize)]
pub struct Photo {
    pub full_size_url: Url,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
    pub profile_url: Url,
}

impl From<User> for crate::marketplace::item::Seller {
    fn from(user: User) -> Self {
        Self::builder().username(user.login).profile_url(user.profile_url).build()
    }
}

#[derive(Debug, Deserialize)]
pub enum Status {
    #[serde(rename = "Not fully functional", alias = "Niet helemaal functioneel")]
    NotFullyFunctional,

    #[serde(alias = "Veelgebruikt")]
    Satisfactory,

    #[serde(alias = "Goed")]
    Good,

    #[serde(rename = "Very good", alias = "Heel goed")]
    VeryGood,

    #[serde(rename = "New without tags", alias = "Nieuw zonder prijskaartje")]
    NewWithoutTags,

    #[serde(rename = "New with tags", alias = "Nieuw met prijskaartje")]
    NewWithTags,
}

impl From<Status> for crate::marketplace::item::Condition {
    fn from(status: Status) -> Self {
        match status {
            Status::NotFullyFunctional => {
                Self::Used(crate::marketplace::item::Used::NotFullyFunctional)
            }
            Status::Satisfactory => Self::Used(crate::marketplace::item::Used::Satisfactory),
            Status::Good => Self::Used(crate::marketplace::item::Used::Good),
            Status::VeryGood => Self::Used(crate::marketplace::item::Used::VeryGood),
            Status::NewWithoutTags => Self::New(crate::marketplace::item::New::WithoutTags),
            Status::NewWithTags => Self::New(crate::marketplace::item::New::WithTags),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_deserialize_item_ok() -> Result {
        // language=json
        let _: Item = serde_json::from_str(
            r##"
            {
              "id": 6245197443,
              "title": "Unifi u6 pro",
              "price": {
                "amount": "135.0",
                "currency_code": "EUR"
              },
              "is_visible": true,
              "discount": null,
              "brand_title": "Ubiquiti",
              "path": "/items/6245197443-unifi-u6-pro",
              "user": {
                "id": 258360251,
                "login": "hertokken",
                "profile_url": "https://www.vinted.nl/member/258360251-hertokken",
                "photo": null,
                "business": false
              },
              "conversion": null,
              "url": "https://www.vinted.nl/items/6245197443-unifi-u6-pro",
              "promoted": false,
              "photo": {
                "id": 25350626557,
                "image_no": 1,
                "width": 600,
                "height": 800,
                "dominant_color": "#B4AFB3",
                "dominant_color_opaque": "#E9E7E8",
                "url": "https://images1.vinted.net/t/03_006f6_dTDx2rZxo5Ma5tTtWH8rW5v6/f800/1746107501.jpeg?s=a770eadce37dc98aa996920000668e0fbfcf180d",
                "is_main": true,
                "thumbnails": [
                  {
                    "type": "thumb70x100",
                    "url": "https://images1.vinted.net/t/03_006f6_dTDx2rZxo5Ma5tTtWH8rW5v6/70x100/1746107501.jpeg?s=47cb0126bc695fdfea2385d6733260dc9e9be634",
                    "width": 70,
                    "height": 100,
                    "original_size": null
                  },
                  {
                    "type": "thumb150x210",
                    "url": "https://images1.vinted.net/t/03_006f6_dTDx2rZxo5Ma5tTtWH8rW5v6/150x210/1746107501.jpeg?s=7aa83c5c7dfe9fe849c12b9964816f0bee8f94b2",
                    "width": 150,
                    "height": 210,
                    "original_size": null
                  },
                  {
                    "type": "thumb310x430",
                    "url": "https://images1.vinted.net/t/03_006f6_dTDx2rZxo5Ma5tTtWH8rW5v6/310x430/1746107501.jpeg?s=a836240144793a273808d7bea7f6ea7844d52c36",
                    "width": 310,
                    "height": 430,
                    "original_size": null
                  },
                  {
                    "type": "thumb428x624",
                    "url": "https://images1.vinted.net/t/03_006f6_dTDx2rZxo5Ma5tTtWH8rW5v6/f800/1746107501.jpeg?s=a770eadce37dc98aa996920000668e0fbfcf180d",
                    "width": 321,
                    "height": 428,
                    "original_size": true
                  },
                  {
                    "type": "thumb624x428",
                    "url": "https://images1.vinted.net/t/03_006f6_dTDx2rZxo5Ma5tTtWH8rW5v6/f800/1746107501.jpeg?s=a770eadce37dc98aa996920000668e0fbfcf180d",
                    "width": 468,
                    "height": 624,
                    "original_size": true
                  },
                  {
                    "type": "thumb364x428",
                    "url": "https://images1.vinted.net/t/03_006f6_dTDx2rZxo5Ma5tTtWH8rW5v6/f800/1746107501.jpeg?s=a770eadce37dc98aa996920000668e0fbfcf180d",
                    "width": 273,
                    "height": 364,
                    "original_size": true
                  }
                ],
                "high_resolution": {
                  "id": "03_006f6_dTDx2rZxo5Ma5tTtWH8rW5v6",
                  "timestamp": 1746107501,
                  "orientation": null
                },
                "is_suspicious": false,
                "full_size_url": "https://images1.vinted.net/tc/03_006f6_dTDx2rZxo5Ma5tTtWH8rW5v6/1746107501.jpeg?s=c9944e399ea422797bdcb1731e09eeb0e4a62d75",
                "is_hidden": false,
                "extra": {}
              },
              "favourite_count": 3,
              "is_favourite": false,
              "badge": null,
              "service_fee": {
                "amount": "7.45",
                "currency_code": "EUR"
              },
              "total_item_price": {
                "amount": "142.45",
                "currency_code": "EUR"
              },
              "view_count": 0,
              "size_title": "",
              "content_source": "search",
              "status": "Heel goed",
              "icon_badges": [],
              "item_box": {
                "first_line": "Ubiquiti",
                "second_line": "Heel goed",
                "exposure": {
                  "test_id": "67037",
                  "test_name": "item_high_demand_signal",
                  "variant": "on",
                  "test_anon_id": "2ca03c9c-7204-4499-82f5-80b6af39af85",
                  "test_user_id": "246887296",
                  "country_code": "NL"
                },
                "accessibility_label": "Unifi u6 pro, merk: Ubiquiti, staat: Heel goed, 135,00 €, 142,45 € inclusief Kopersbescherming",
                "badge": {
                  "title": "Populair"
                }
              },
              "search_tracking_params": {
                "score": -4.771793842315674,
                "matched_queries": []
              }
            }"##,
        )?;
        Ok(())
    }
}
