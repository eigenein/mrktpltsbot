use clap::Parser;
use futures::{StreamExt, TryStreamExt, stream};

use crate::{
    cli::Cli,
    client::Client,
    db::Db,
    marktplaats::Marktplaats,
    prelude::*,
    telegram::{Telegram, methods::Method},
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
    let client = Client::new()?;
    let telegram = Telegram::new(client.clone(), cli.bot_token.into())?;
    let marktplaats = Marktplaats(client);
    let db = Db::try_new(&cli.db).await?;

    let telegram_updates = telegram
        .clone()
        .into_updates(0, cli.telegram_poll_timeout_secs);
    let telegram_reactor = bot::telegram::Reactor::builder()
        .authorized_chat_ids(cli.authorized_chat_ids.into_iter().collect())
        .db(&db)
        .marktplaats(&marktplaats)
        .command_builder(bot::telegram::try_init(&telegram).await?)
        .build();
    let marktplaats_reactor = bot::marktplaats::Reactor::builder()
        .db(&db)
        .marktplaats(&marktplaats)
        .build();
    telegram_reactor
        .run(telegram_updates)
        .map_ok(|reactions| stream::iter(reactions).map(Ok))
        .try_flatten()
        .chain(marktplaats_reactor.run())
        .try_for_each(|reaction| {
            let telegram = &telegram;
            async move { reaction.call_discarded_on(telegram).await }
        })
        .await
        .context("reactor error")
}
