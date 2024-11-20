#![doc = include_str!("../README.md")]

use std::{pin::pin, time::Duration};

use clap::Parser;
use futures::{StreamExt, TryFutureExt};

use crate::{
    cli::Args,
    client::Client,
    db::{Db, notification::Notifications},
    heartbeat::Heartbeat,
    marktplaats::Marktplaats,
    prelude::*,
    telegram::{Telegram, reaction::Reaction},
};

mod cli;
mod client;
mod db;
mod heartbeat;
mod logging;
mod marktplaats;
mod prelude;
mod serde;
mod telegram;

fn main() -> Result {
    let dotenv_result = dotenvy::dotenv();
    let cli = Args::parse();
    let _tracing_guards = logging::init(cli.sentry_dsn.as_deref())?;
    if let Err(error) = dotenv_result {
        warn!("Could not load `.env`: {error:#}");
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async_main(cli))
        .inspect_err(|error| error!("Fatal error: {error:#}"))
}

async fn async_main(cli: Args) -> Result {
    let client = Client::try_new()?;
    let telegram = Telegram::new(client.clone(), cli.telegram.bot_token.into())?;
    let marktplaats = Marktplaats(client.clone());
    let db = Db::try_new(&cli.db).await?;
    let command_builder = telegram::bot::try_init(&telegram).await?;

    // Handle Telegram updates:
    let telegram_heartbeat = Heartbeat::new(&client, cli.telegram.heartbeat_url);
    let telegram_updates =
        telegram.clone().into_updates(0, cli.telegram.poll_timeout_secs, &telegram_heartbeat);
    let telegram_reactor = telegram::bot::Reactor::builder()
        .authorized_chat_ids(cli.telegram.authorized_chat_ids.into_iter().collect())
        .db(&db)
        .marktplaats(&marktplaats)
        .command_builder(&command_builder)
        .build();
    let telegram_reactions = telegram_reactor.run(telegram_updates);

    // Handle Marktplaats subscriptions:
    let marktplaats_reactor = marktplaats::bot::Reactor::builder()
        .db(&db)
        .marktplaats(&marktplaats)
        .crawl_interval(Duration::from_secs(cli.marktplaats.crawl_interval_secs))
        .command_builder(&command_builder)
        .search_limit(cli.marktplaats.search_limit)
        .build();
    let marktplaats_heartbeat = Heartbeat::new(&client, cli.marktplaats.heartbeat_url);
    let marktplaats_reactions = marktplaats_reactor.run(&marktplaats_heartbeat);

    // Now, merge all the reactions and execute them:
    let reactor = tokio_stream::StreamExt::merge(telegram_reactions, marktplaats_reactions)
        .filter_map(|result| async {
            // Log and skip error to keep the reactor going.
            result.inspect_err(|error| error!("Reactor error: {error:#}")).ok()
        })
        .for_each(|reaction| execute_reaction(reaction, &telegram, &db));
    pin!(reactor).await;

    unreachable!()
}

/// Execute the reaction.
///
/// This is infallible since it must not stop the entire reactor.
async fn execute_reaction(reaction: Reaction<'_>, telegram: &Telegram, db: &Db) {
    let result = reaction
        .react_to(telegram)
        .and_then(|()| async {
            if let Some(notification) = &reaction.notification {
                Notifications(&mut *db.connection().await).upsert(notification).await?;
            }
            Ok(())
        })
        .await;
    if let Err(error) = result {
        error!("Failed to execute the reaction: {error:#}");
    }
}
