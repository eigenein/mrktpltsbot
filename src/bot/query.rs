/// User's search query.
pub struct SearchQuery {
    pub text: String,

    /// SeaHash-ed text.
    ///
    /// Used instead of the text where the payload size is limited.
    pub hash: i64,
}

impl From<&str> for SearchQuery {
    fn from(text: &str) -> Self {
        let text = text.trim().to_lowercase();
        Self {
            #[expect(clippy::cast_possible_wrap)]
            hash: seahash::hash(text.as_bytes()) as i64,

            text,
        }
    }
}
