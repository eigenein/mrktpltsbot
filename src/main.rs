use std::time::Duration;

use clap::Parser;
use futures::TryStreamExt;

use crate::{
    cli::Args,
    client::Client,
    db::{Db, notification::Notifications},
    marktplaats::Marktplaats,
    prelude::*,
    telegram::{Telegram, reaction::Reaction},
};

mod cli;
mod client;
mod db;
mod logging;
mod marktplaats;
mod prelude;
mod serde;
mod telegram;

fn main() -> Result {
    let cli = Args::parse();
    let _tracing_guards = logging::init(cli.sentry_dsn.as_deref())?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async_main(cli))
        .inspect_err(|error| error!("Fatal error: {error:#}"))
}

async fn async_main(cli: Args) -> Result {
    let client = Client::new()?;
    let telegram = Telegram::new(client.clone(), cli.telegram.bot_token.into())?;
    let marktplaats = Marktplaats(client);
    let db = Db::try_new(&cli.db).await?;
    let command_builder = telegram::bot::try_init(&telegram).await?;

    // Handle Telegram updates:
    let telegram_updates = telegram
        .clone()
        .into_updates(0, cli.telegram.poll_timeout_secs);
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
    let marktplaats_reactions = marktplaats_reactor.run();

    // Now, merge all the reactions and execute them:
    tokio_stream::StreamExt::merge(telegram_reactions, marktplaats_reactions)
        .try_for_each(|reaction| execute_reaction(reaction, &telegram, &db))
        .await
        .context("reactor error")
}

async fn execute_reaction(reaction: Reaction<'_>, telegram: &Telegram, db: &Db) -> Result {
    reaction
        .react_to(telegram)
        .await
        .context("failed to execute the reaction")?;
    if let Some(notification) = &reaction.notification {
        Notifications(&mut *db.connection().await)
            .upsert(notification)
            .await?;
    }
    Ok(())
}
