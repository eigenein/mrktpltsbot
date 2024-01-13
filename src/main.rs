#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use clap::Parser;

use crate::{cli::Cli, prelude::*};

mod cli;
mod prelude;
mod tracing;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let _tracing_guards = tracing::init(cli.sentry_dsn, cli.traces_sample_rate)?;
    Ok(())
}
