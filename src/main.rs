use clap::Parser;

use crate::{
    bot::Bot,
    cli::Cli,
    client::Client,
    db::Db,
    marktplaats::Marktplaats,
    prelude::*,
    telegram::Telegram,
};

mod bot;
mod cli;
mod client;
pub mod db;
mod logging;
mod marktplaats;
mod prelude;
mod serde;
mod telegram;

#[tokio::main]
async fn main() -> Result {
    let cli = Cli::parse();
    let _tracing_guards = logging::init(cli.sentry_dsn.as_deref())?;
    fallible_main(cli)
        .await
        .inspect_err(|error| error!("Fatal error: {error:#}"))
}

async fn fallible_main(cli: Cli) -> Result {
    let db = Db::new(&cli.db).await?;
    let client = Client::new()?;
    let marktplaats = Marktplaats(client.clone());
    let telegram = Telegram::new(client, cli.bot_token)?;

    Bot::builder()
        .telegram(telegram)
        .marktplaats(marktplaats)
        .db(db)
        .poll_timeout_secs(cli.timeout_secs)
        .build()
        .run_telegram()
        .await
        .context("fatal error")
}
