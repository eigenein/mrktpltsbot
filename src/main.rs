use clap::Parser;

use crate::{
    bot::Bot,
    cli::Cli,
    client::Client,
    db::Db,
    marktplaats::Marktplaats,
    prelude::*,
    telegram::{Telegram, objects::ChatId},
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

fn main() -> Result {
    let cli = Cli::parse();
    let _tracing_guards = logging::init(cli.sentry_dsn.as_deref())?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async_main(cli))
        .inspect_err(|error| error!("Fatal error: {error:#}"))
}

async fn async_main(cli: Cli) -> Result {
    let db = Db::new(&cli.db).await?;
    let client = Client::new()?;
    let marktplaats = Marktplaats(client.clone());
    let telegram = Telegram::new(client, cli.bot_token.into())?;
    let authorized_chat_ids = cli
        .authorized_chat_ids
        .iter()
        .map(|chat_id| ChatId::Integer(*chat_id))
        .collect();
    Bot::builder()
        .telegram(telegram)
        .authorized_chat_ids(authorized_chat_ids)
        .marktplaats(marktplaats)
        .db(db)
        .poll_timeout_secs(cli.timeout_secs)
        .offset(0)
        .try_connect()
        .await?
        .run_telegram()
        .await
        .context("fatal error")
}
