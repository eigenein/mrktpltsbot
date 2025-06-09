#![doc = include_str!("../README.md")]

use std::time::Duration;

use clap::Parser;
use reqwest_middleware::ClientWithMiddleware;
use secrecy::ExposeSecret;

use crate::{
    cli::{Args, Command, RunArgs, VintedCommand},
    db::{Db, KeyValues},
    heartbeat::Heartbeat,
    logging::Logging,
    marketplace::{
        Marktplaats,
        MarktplaatsClient,
        SearchBot,
        Vinted,
        VintedAuthenticationTokens,
        VintedClient,
    },
    prelude::*,
    telegram::{Telegram, TelegramBot},
};

mod cli;
mod client;
mod db;
mod heartbeat;
mod logging;
mod marketplace;
mod prelude;
mod serde;
mod telegram;

fn main() -> Result {
    let dotenv_result = dotenvy::dotenv();
    let cli = Args::parse();
    let logging = Logging::init(cli.sentry_dsn.as_deref())?;
    if let Err(error) = dotenv_result {
        log::warn!("⚠️ Could not load `.env`: {error:#}");
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async_main(cli))
        .inspect_err(|error| {
            capture_anyhow(error);
        })?;
    logging.try_shutdown()
}

async fn async_main(cli: Args) -> Result {
    let db = Db::try_new(&cli.db).await?;
    let client = client::try_new(cli.trace_requests)?;
    match cli.command {
        Command::Run(args) => run(db, client, *args).await,
        Command::Vinted { command } => manage_vinted(db, client, command).await,
    }
}

/// Run the bot indefinitely.
async fn run(db: Db, client: ClientWithMiddleware, args: RunArgs) -> Result {
    let telegram = Telegram::new(client.clone(), args.telegram.bot_token.into())?;
    let command_builder = telegram.command_builder().await?;

    // Marktplaats connection:
    let marktplaats = Marktplaats::builder()
        .client(MarktplaatsClient(client.clone()))
        .search_limit(args.marktplaats.marktplaats_search_limit)
        .search_in_title_and_description(args.marktplaats.search_in_title_and_description)
        .heartbeat(Heartbeat::new(client.clone(), args.marktplaats.heartbeat_url))
        .build();

    // Vinted connection:
    let vinted = Vinted::builder()
        .client(VintedClient(client.clone()))
        .search_limit(args.vinted.vinted_search_limit)
        .db(db.clone())
        .heartbeat(Heartbeat::new(client.clone(), args.vinted.heartbeat_url))
        .build();

    // Telegram bot:
    let telegram_bot = TelegramBot::builder()
        .telegram(telegram.clone())
        .authorized_chat_ids(args.telegram.authorized_chat_ids.into_iter().collect())
        .db(db.clone())
        .marktplaats(marktplaats.clone())
        .vinted(vinted.clone())
        .poll_timeout_secs(args.telegram.poll_timeout_secs)
        .heartbeat(Heartbeat::new(client, args.telegram.heartbeat_url))
        .command_builder(command_builder.clone())
        .try_init()
        .await?;

    // Search bot:
    let search_bot = SearchBot::builder()
        .db(db)
        .search_interval(Duration::from_secs(args.search_interval_secs))
        .marktplaats(marktplaats)
        .vinted(vinted)
        .telegram(telegram)
        .command_builder(command_builder)
        .build();

    // Run the bots:
    tokio::try_join!(tokio::spawn(telegram_bot.run()), tokio::spawn(search_bot.run()))?;
    Ok(())
}

/// Manage Vinted settings.
async fn manage_vinted(db: Db, client: ClientWithMiddleware, command: VintedCommand) -> Result {
    match command {
        VintedCommand::Authenticate { refresh_token } => {
            let tokens = VintedClient(client).refresh_token(refresh_token.expose_secret()).await?;
            KeyValues(&mut *db.connection().await).upsert(&tokens).await?;
            info!("✅ Succeeded, now the bot will search on Vinted as well");
        }

        VintedCommand::ShowTokens => {
            let tokens: Option<VintedAuthenticationTokens> =
                KeyValues(&mut *db.connection().await).fetch().await?;
            match tokens {
                Some(tokens) => {
                    info!("🔑", tokens.access = tokens.access, tokens.refresh = tokens.refresh);
                }
                None => {
                    info!("🔒 There are no stored tokens");
                }
            }
        }
    }
    Ok(())
}
