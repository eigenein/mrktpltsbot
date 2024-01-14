pub use anyhow::Context;
pub use tracing::{debug, error, info, warn};

pub type Result<T = ()> = anyhow::Result<T>;
pub type Error = anyhow::Error;
