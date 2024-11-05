use std::fmt::{Debug, Display, Formatter};

use sqlx::{Database, Decode, Encode, Sqlite, Type, encode::IsNull, error::BoxDynError};

/// [SeaHash][1] of a search query.
///
/// Used instead of the text where the payload size is limited (e.g. in `/start` payload).
///
/// [1]: https://docs.rs/seahash/latest/seahash/
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct QueryHash(pub u64);

impl Display for QueryHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<&str> for QueryHash {
    /// Calculate hash from a text.
    fn from(text: &str) -> Self {
        Self(seahash::hash(text.as_bytes()))
    }
}

impl Type<Sqlite> for QueryHash {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <i64 as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for QueryHash {
    #[expect(clippy::cast_possible_wrap)]
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <i64 as Encode<'_, Sqlite>>::encode_by_ref(&(self.0 as i64), buf)
    }

    fn size_hint(&self) -> usize {
        size_of_val(&self.0)
    }
}

impl<'q> Decode<'q, Sqlite> for QueryHash {
    #[expect(clippy::cast_sign_loss)]
    fn decode(value: <Sqlite as Database>::ValueRef<'q>) -> Result<Self, BoxDynError> {
        Ok(Self(<i64 as Decode<'_, Sqlite>>::decode(value)? as u64))
    }
}
