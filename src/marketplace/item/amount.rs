use std::str::FromStr;

use maud::{Markup, Render, html};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};

use crate::prelude::*;

/// Monetary amount.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Amount(pub Decimal);

impl Amount {
    pub const ZERO: Self = Self(Decimal::ZERO);

    #[expect(dead_code)]
    pub fn deserialize_from_string<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        match Decimal::from_str(&text) {
            Ok(amount) => Ok(Self(amount)),
            Err(error) => Err(serde::de::Error::custom(format!(
                "failed to deserialize a monetary amount: {error:#}"
            ))),
        }
    }

    pub fn deserialize_from_cents<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(Decimal::new(i64::deserialize(deserializer)?, 2)))
    }
}

impl Render for Amount {
    fn render(&self) -> Markup {
        html! {
            // TODO: support other currencies.
            "€" (self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn price_amount_deserialize_from_cents_ok() -> Result {
        #[derive(Deserialize)]
        struct Item {
            #[serde(deserialize_with = "Amount::deserialize_from_cents")]
            amount: Amount,
        }

        // language=json
        let item: Item = serde_json::from_str(r#"{"amount": 1234}"#)?;
        assert_eq!(item.amount.0, dec!(12.34));

        Ok(())
    }
}
