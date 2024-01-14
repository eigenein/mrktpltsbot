#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::time::Duration;

use clap::Parser;
use sentry::SessionMode;

use crate::{
    cli::{Cli, Command},
    crawler::Crawler,
    prelude::*,
};

mod cli;
mod crawler;
mod marktplaats;
mod prelude;
mod throttler;
mod tracing;

#[tokio::main]
async fn main() -> Result {
    if dotenvy::dotenv().is_err() {
        warn!("failed to load `.env`");
    }
    let cli = Cli::parse();
    match cli.command {
        Command::Crawler => {
            let _tracing_guards =
                tracing::init(cli.sentry_dsn, SessionMode::Application, cli.traces_sample_rate)?;
            Crawler::new(Duration::from_millis(200))?.run().await
        }
    }
}
