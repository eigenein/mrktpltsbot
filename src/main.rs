use clap::Parser;
use futures::{StreamExt, TryFutureExt, TryStreamExt, stream};

use crate::{
    bot::telegram::Reactor,
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

    Reactor::builder()
        .authorized_chat_ids(cli.authorized_chat_ids.into_iter().collect())
        .db(Db::new(&cli.db).await?)
        .marktplaats(Marktplaats(client))
        .command_builder(bot::telegram::try_init(&telegram).await?)
        .build()
        .run(
            telegram
                .clone()
                .into_updates(0, cli.telegram_poll_timeout_secs),
        )
        .map_ok(|reactions| stream::iter(reactions).map(Ok))
        .try_flatten()
        .try_for_each(|reaction| {
            let telegram = &telegram;
            async move { reaction.call_discarded_on(telegram).await }
        })
        .inspect_err(|error| error!("reactor error: {error:#}"))
        .await?;

    Ok(())
}
