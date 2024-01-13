#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use clap::Parser;

use crate::{
    cli::{Cli, Command},
    crawler::Crawler,
    prelude::*,
};

mod cli;
mod crawler;
mod marktplaats;
mod prelude;
mod tracing;

#[tokio::main]
async fn main() -> Result {
    if dotenvy::dotenv().is_err() {
        warn!("failed to load `.env`");
    }
    let cli = Cli::parse();
    let _tracing_guards = tracing::init(cli.sentry_dsn, cli.traces_sample_rate)?;
    match cli.command {
        Command::Crawler => Crawler::new()?.run().await,
    }
}
