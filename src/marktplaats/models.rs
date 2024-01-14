use chrono::{DateTime, Local};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchResponse {
    pub listings: Vec<Listing>,
}

#[derive(Deserialize)]
pub struct Listing {
    #[serde(rename = "itemId")]
    pub item_id: String,

    pub title: String,

    pub description: String,

    #[serde(rename = "date")]
    pub timestamp: DateTime<Local>,

    #[serde(rename = "priceInfo")]
    pub price: PriceInfo,

    #[serde(rename = "pictures", default)]
    pub pictures: Vec<Picture>,
}

#[derive(Deserialize)]
pub struct PriceInfo {
    #[serde(rename = "priceCents")]
    pub cents: u32,

    #[serde(rename = "priceType")]
    pub type_: PriceType,
}

#[derive(Deserialize)]
pub enum PriceType {
    /// Fixed price, bidding are not allowed.
    #[serde(rename = "FIXED")]
    Fixed,

    #[serde(rename = "ON_REQUEST")]
    OnRequest,

    /// Price, bids are allowed.
    #[serde(rename = "MIN_BID")]
    MinBid,

    #[serde(rename = "SEE_DESCRIPTION")]
    SeeDescription,

    #[serde(rename = "NOTK")]
    ToBeAgreed,

    #[serde(rename = "RESERVED")]
    Reserved,

    /// No asking price, only bidding.
    #[serde(rename = "FAST_BID")]
    FastBid,

    #[serde(rename = "FREE")]
    Free,

    #[serde(rename = "EXCHANGE")]
    Exchange,
}

#[derive(Deserialize)]
pub enum PriorityProduct {
    #[serde(rename = "NONE")]
    None,

    #[serde(rename = "DAGTOPPER")]
    DayTopper,
}

#[derive(Deserialize)]
pub struct Picture {
    pub url: String,

    #[serde(rename = "extraSmallUrl")]
    pub extra_small_url: String,

    #[serde(rename = "mediumUrl")]
    pub medium_url: String,

    #[serde(rename = "largeUrl")]
    pub large_url: String,

    #[serde(rename = "extraExtraLargeUrl")]
    pub extra_extra_large_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn parse_m2068620907_ok() -> Result {
        let response: SearchResponse = serde_json::from_str(
            // language=json
            r#"{"listings":[{"itemId":"m2068620907","title":"raspmachine /reibemaschine Alexanderwerk 1050 antiek","description":"Raspmachine handmolen \"alexanderwerk\" nr. 1050 Overgrootmoeders handmolen: grote rood gelakte handmolen met \"alexan","categorySpecificDescription":"Raspmachine handmolen \"alexanderwerk\" nr. 1050 Overgrootmoeders handmolen: grote rood gelakte handmolen met \"alexanderwerk\",  reibemaschine erop. Aan de voorzijde boven een logo in de vorm van een engelengezichtje met daarachter v...","thinContent":false,"priceInfo":{"priceCents":3500,"priceType":"MIN_BID"},"location":{"cityName":"IJmuiden","countryName":"Nederland","countryAbbreviation":"NL","distanceMeters":-1000,"isBuyerLocation":false,"onCountryLevel":false,"abroad":false,"latitude":52.462357302267,"longitude":4.6164179342887},"date":"2024-01-12T21:44:22Z","imageUrls":["//images.marktplaats.com/api/v1/listing-mp-p/images/b2/b20cb3cb-5882-4b89-9617-10971b93b655?rule=ecg_mp_eps$_82.jpg"],"sellerInformation":{"sellerId":9848530,"sellerName":"soraya","showSoiUrl":true,"showWebsiteUrl":false,"isVerified":false},"categoryId":1842,"priorityProduct":"NONE","videoOnVip":false,"urgencyFeatureActive":false,"napAvailable":false,"attributes":[{"key":"delivery","value":"Ophalen of Verzenden","values":["Ophalen of Verzenden"]}],"extendedAttributes":[{"key":"delivery","value":"Ophalen of Verzenden","values":["Ophalen of Verzenden"]}],"traits":["PACKAGE_FREE"],"verticals":["antique_kitchen_supplies","antique_and_art"],"pictures":[{"id":9347484131,"mediaId":"","url":"https://images.marktplaats.com/api/v1/listing-mp-p/images/b2/b20cb3cb-5882-4b89-9617-10971b93b655?rule=ecg_mp_eps$_#.jpg","extraSmallUrl":"https://images.marktplaats.com/api/v1/listing-mp-p/images/b2/b20cb3cb-5882-4b89-9617-10971b93b655?rule=ecg_mp_eps$_14.jpg","mediumUrl":"https://images.marktplaats.com/api/v1/listing-mp-p/images/b2/b20cb3cb-5882-4b89-9617-10971b93b655?rule=ecg_mp_eps$_82.jpg","largeUrl":"https://images.marktplaats.com/api/v1/listing-mp-p/images/b2/b20cb3cb-5882-4b89-9617-10971b93b655?rule=ecg_mp_eps$_83.jpg","extraExtraLargeUrl":"https://images.marktplaats.com/api/v1/listing-mp-p/images/b2/b20cb3cb-5882-4b89-9617-10971b93b655?rule=ecg_mp_eps$_85.jpg","aspectRatio":{"width":3,"height":4}}],"vipUrl":"/v/antiek-en-kunst/antiek-keukenbenodigdheden/m2068620907-raspmachine-reibemaschine-alexanderwerk-1050-antiek"}],"topBlock":[],"facets":[{"key":"PriceCents","type":"AttributeRangeFacet"},{"key":"RelevantCategories","type":"CategoryTreeFacet","categories":[{"id":1,"selected":false,"isValuableForSeo":true,"dominant":false,"label":"Antiek en Kunst","key":"antiek-en-kunst","parentId":null,"parentKey":false},{"id":1842,"histogramCount":1,"selected":false,"isValuableForSeo":true,"dominant":false,"label":"Antiek | Keukenbenodigdheden","key":"antiek-keukenbenodigdheden","parentId":1,"parentKey":"antiek-en-kunst"}]},{"id":1627,"key":"condition","type":"AttributeGroupFacet","label":"Conditie","attributeGroup":[{"attributeValueKey":"Nieuw","attributeValueId":30,"attributeValueLabel":"Nieuw","selected":false,"isValuableForSeo":false},{"attributeValueKey":"Zo goed als nieuw","attributeValueId":31,"attributeValueLabel":"Zo goed als nieuw","selected":false,"isValuableForSeo":false},{"attributeValueKey":"Gebruikt","attributeValueId":32,"attributeValueLabel":"Gebruikt","selected":false,"isValuableForSeo":false},{"attributeValueKey":"Niet werkend","attributeValueId":13940,"attributeValueLabel":"Niet werkend","selected":false,"isValuableForSeo":false}],"singleSelect":false,"categoryId":0},{"id":8,"key":"delivery","type":"AttributeGroupFacet","label":"Levering","attributeGroup":[{"attributeValueKey":"Ophalen","attributeValueId":33,"attributeValueLabel":"Ophalen","histogramCount":1,"selected":false,"isValuableForSeo":false},{"attributeValueKey":"Verzenden","attributeValueId":34,"attributeValueLabel":"Verzenden","histogramCount":1,"selected":false,"isValuableForSeo":false}],"singleSelect":false,"categoryId":0},{"id":987654321,"key":"offeredSince","type":"AttributeGroupFacet","label":"Aangeboden sinds","attributeGroup":[{"attributeValueKey":"Vandaag","selected":false,"isValuableForSeo":false,"default":false},{"attributeValueKey":"Gisteren","selected":false,"isValuableForSeo":false,"default":false},{"attributeValueKey":"Een week","histogramCount":1,"selected":false,"isValuableForSeo":false,"default":false},{"attributeValueKey":"Altijd","histogramCount":1,"selected":true,"isValuableForSeo":false,"default":true}],"singleSelect":true,"categoryId":0}],"totalResultCount":1,"maxAllowedPageNumber":2,"correlationId":"eb9a5799-4723-4d20-9c26-1ef13ee61864","originalQuery":"m2068620907","sortOptions":[{"sortBy":"OPTIMIZED","sortOrder":"DECREASING"},{"sortBy":"SORT_INDEX","sortOrder":"DECREASING"},{"sortBy":"SORT_INDEX","sortOrder":"INCREASING"},{"sortBy":"PRICE","sortOrder":"INCREASING"},{"sortBy":"PRICE","sortOrder":"DECREASING"}],"isSearchSaved":false,"hasErrors":false,"alternativeLocales":[],"searchRequest":{"originalRequest":{"categories":{},"searchQuery":"m2068620907","attributes":{},"attributesById":[],"attributesByKey":[],"attributeRanges":[],"attributeLabels":[],"sortOptions":{"sortBy":"","sortOrder":"","sortAttribute":""},"pagination":{"offset":0,"limit":1},"distance":{"postcode":""},"viewOptions":{"kind":"list-view"},"bypassSpellingSuggestion":false},"categories":{},"searchQuery":"m2068620907","attributes":{},"attributesById":[],"attributesByKey":[],"attributeRanges":[],"attributeLabels":[],"sortOptions":{"sortBy":"","sortOrder":"","sortAttribute":""},"pagination":{"offset":0,"limit":1},"distance":{"postcode":""},"viewOptions":{"kind":"list-view"},"bypassSpellingSuggestion":false},"searchCategory":0,"searchCategoryOptions":[{"fullName":"Antiek en Kunst","id":1,"key":"antiek-en-kunst","name":"Antiek en Kunst"},{"fullName":"Audio, Tv en Foto","id":31,"key":"audio-tv-en-foto","name":"Audio, Tv en Foto"},{"fullName":"Auto's","id":91,"key":"auto-s","name":"Auto's"},{"fullName":"Auto-onderdelen","id":2600,"key":"auto-onderdelen","name":"Auto-onderdelen"},{"fullName":"Auto diversen","id":48,"key":"auto-diversen","name":"Auto diversen"},{"fullName":"Boeken","id":201,"key":"boeken","name":"Boeken"},{"fullName":"Caravans en Kamperen","id":289,"key":"caravans-en-kamperen","name":"Caravans en Kamperen"},{"fullName":"Cd's en Dvd's","id":1744,"key":"cd-s-en-dvd-s","name":"Cd's en Dvd's"},{"fullName":"Computers en Software","id":322,"key":"computers-en-software","name":"Computers en Software"},{"fullName":"Contacten en Berichten","id":378,"key":"contacten-en-berichten","name":"Contacten en Berichten"},{"fullName":"Diensten en Vakmensen","id":1098,"key":"diensten-en-vakmensen","name":"Diensten en Vakmensen"},{"fullName":"Dieren en Toebehoren","id":395,"key":"dieren-en-toebehoren","name":"Dieren en Toebehoren"},{"fullName":"Doe-het-zelf en Verbouw","id":239,"key":"doe-het-zelf-en-verbouw","name":"Doe-het-zelf en Verbouw"},{"fullName":"Fietsen en Brommers","id":445,"key":"fietsen-en-brommers","name":"Fietsen en Brommers"},{"fullName":"Hobby en Vrije tijd","id":1099,"key":"hobby-en-vrije-tijd","name":"Hobby en Vrije tijd"},{"fullName":"Huis en Inrichting","id":504,"key":"huis-en-inrichting","name":"Huis en Inrichting"},{"fullName":"Huizen en Kamers","id":1032,"key":"huizen-en-kamers","name":"Huizen en Kamers"},{"fullName":"Kinderen en Baby's","id":565,"key":"kinderen-en-baby-s","name":"Kinderen en Baby's"},{"fullName":"Kleding | Dames","id":621,"key":"kleding-dames","name":"Kleding | Dames"},{"fullName":"Kleding | Heren","id":1776,"key":"kleding-heren","name":"Kleding | Heren"},{"fullName":"Motoren","id":678,"key":"motoren","name":"Motoren"},{"fullName":"Muziek en Instrumenten","id":728,"key":"muziek-en-instrumenten","name":"Muziek en Instrumenten"},{"fullName":"Postzegels en Munten","id":1784,"key":"postzegels-en-munten","name":"Postzegels en Munten"},{"fullName":"Sieraden, Tassen en Uiterlijk","id":1826,"key":"sieraden-tassen-en-uiterlijk","name":"Sieraden en Tassen"},{"fullName":"Spelcomputers en Games","id":356,"key":"spelcomputers-en-games","name":"Spelcomputers, Games"},{"fullName":"Sport en Fitness","id":784,"key":"sport-en-fitness","name":"Sport en Fitness"},{"fullName":"Telecommunicatie","id":820,"key":"telecommunicatie","name":"Telecommunicatie"},{"fullName":"Tickets en Kaartjes","id":1984,"key":"tickets-en-kaartjes","name":"Tickets en Kaartjes"},{"fullName":"Tuin en Terras","id":1847,"key":"tuin-en-terras","name":"Tuin en Terras"},{"fullName":"Vacatures","id":167,"key":"vacatures","name":"Vacatures"},{"fullName":"Vakantie","id":856,"key":"vakantie","name":"Vakantie"},{"fullName":"Verzamelen","id":895,"key":"verzamelen","name":"Verzamelen"},{"fullName":"Watersport en Boten","id":976,"key":"watersport-en-boten","name":"Watersport en Boten"},{"fullName":"Witgoed en Apparatuur","id":537,"key":"witgoed-en-apparatuur","name":"Witgoed en Apparatuur"},{"fullName":"Zakelijke goederen","id":1085,"key":"zakelijke-goederen","name":"Zakelijke goederen"},{"fullName":"Diversen","id":428,"key":"diversen","name":"Diversen"}],"seoFriendlyAttributes":[],"seoFriendlyTextAttributes":{},"attributeHierarchy":{"offeredSince":[{"attributeValueId":null,"attributeValueLabel":null,"attributeValueKey":"Altijd","attributeLabel":"Aangeboden sinds","isDefault":true}]},"categoriesById":{},"metaTags":{"metaTitle":"≥ Vind m2068620907 op Marktplaats - januari 2024","metaDescription":"1 aanbiedingen in januari - Koop en verkoop m2068620907 eenvoudig op Marktplaats ✅ Lokale aanbiedingen - Ga ervoor!","pageTitleH1":"<span>Je hebt gezocht op </span><h1>m2068620907</h1>."}}"#,
        )?;
        assert_eq!(response.listings[0].pictures.len(), 1);
        Ok(())
    }
}
