#![allow(unused_imports)]

pub use anyhow::{anyhow, bail, Context, Error};
pub use tracing::{debug, error, info, instrument, trace, warn, Level};

pub type Result<T = (), E = Error> = anyhow::Result<T, E>;
