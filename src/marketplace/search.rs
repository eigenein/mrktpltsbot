use std::{borrow::Cow, collections::BTreeSet};

use deunicode::deunicode;
use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct NormalisedQuery {
    include: BTreeSet<String>,
    exclude: BTreeSet<String>,
}

impl NormalisedQuery {
    pub fn normalise_token(token: &str) -> String {
        deunicode(&token.to_lowercase())
    }

    pub fn parse(text: &str) -> Self {
        let mut this = Self { include: BTreeSet::new(), exclude: BTreeSet::new() };
        for token in text.split_whitespace().map(Self::normalise_token).sorted() {
            if let Some(token) = token.strip_prefix('-') {
                this.exclude.insert(token.to_string());
            } else {
                this.include.insert(token);
            }
        }
        this
    }

    /// Generate search text for marketplace from the included tokens.
    pub fn to_search_text(&self) -> String {
        self.include.iter().join(" ")
    }

    pub fn unparse(&self) -> String {
        let positive = self.include.iter().map(Cow::Borrowed);
        let negative = self.exclude.iter().map(|token| Cow::<String>::Owned(format!("-{token}")));
        positive.chain(negative).join(" ")
    }

    pub fn matches<'a>(&self, terms: impl IntoIterator<Item = &'a str>) -> bool {
        let terms: BTreeSet<_> = terms.into_iter().map(Self::normalise_token).collect();
        self.include.is_subset(&terms) && self.exclude.is_disjoint(&terms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ok() {
        let query = NormalisedQuery::parse("-samsung smartphone");
        assert_eq!(query.include.iter().collect_vec(), &["smartphone"]);
        assert_eq!(query.exclude.iter().collect_vec(), &["samsung"]);
    }

    #[test]
    fn unparse_ok() {
        let query = NormalisedQuery::parse("-samsung smartphone");
        assert_eq!(query.unparse(), "smartphone -samsung");
    }

    #[test]
    fn deunicode_ok() {
        let query = NormalisedQuery::parse("SKÅDIS");
        assert_eq!(query.include.iter().collect_vec(), &["skadis"]);
    }

    #[test]
    fn search_text_ok() {
        let query = NormalisedQuery::parse("-samsung smartphone");
        assert_eq!(query.to_search_text(), "smartphone");
    }

    #[test]
    fn matches_ok() {
        let query = NormalisedQuery::parse("-samsung foldable smartphone");
        assert!(
            query.matches("Xiaomi Foldable Smartphone".split_whitespace()),
            "contains all the positives and no negatives"
        );
        assert!(
            !query.matches("Samsung Foldable Smartphone".split_whitespace()),
            "contains all the positives but also the negative"
        );
        assert!(
            !query.matches("xiaomi smartphone".split_whitespace()),
            "does not contain all the positives"
        );
    }
}
