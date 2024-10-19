/// User's search query.
#[derive(Copy, Clone)]
pub struct SearchQuery<'a> {
    pub text: &'a str,

    /// SeaHash-ed text.
    ///
    /// Used instead of the text where the payload size is limited.
    pub hash: i64,
}

impl<'a> From<&'a str> for SearchQuery<'a> {
    fn from(text: &'a str) -> Self {
        Self {
            text,

            #[expect(clippy::cast_possible_wrap)]
            hash: seahash::hash(text.as_bytes()) as i64,
        }
    }
}
