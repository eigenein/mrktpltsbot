use serde::{Serialize, Serializer};

pub fn as_inner_json<I: Serialize, S: Serializer>(
    inner: &I,
    serializer: S,
) -> crate::prelude::Result<S::Ok, S::Error> {
    let json = serde_json::to_string(inner)
        .map_err(|error| serde::ser::Error::custom(format!("{error:#}")))?;
    serializer.serialize_str(&json)
}
