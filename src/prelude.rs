#![allow(unused_imports)]

pub use anyhow::{anyhow, Context, Error};
pub use tracing::{debug, error, info, instrument, warn};

pub type Result<T = (), E = Error> = anyhow::Result<T, E>;
