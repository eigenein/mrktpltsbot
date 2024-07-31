use clap::Parser;

use crate::{
    cli::{Cli, Command},
    client::build_client,
    marktplaats::Marktplaats,
    prelude::*,
    telegram::{
        requests::{GetMe, GetUpdates},
        Telegram,
    },
};

mod cli;
mod client;
mod logging;
mod marktplaats;
mod prelude;
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
    let client = build_client()?;

    match cli.command {
        Command::QuickSearch { query, limit } => {
            quick_search(&Marktplaats(client), &query, limit).await
        }

        Command::GetMe => {
            let user = Telegram::new(client, cli.bot_token).call(GetMe).await?;
            info!(user.id, user.username);
            Ok(())
        }

        Command::GetUpdates {
            offset,
            limit,
            timeout_secs,
            allowed_updates,
        } => {
            let request = GetUpdates {
                offset,
                limit,
                timeout_secs,
                allowed_updates,
            };
            get_updates(&Telegram::new(client, cli.bot_token), request).await
        }
    }
}

#[instrument(skip_all)]
async fn quick_search(marktplaats: &Marktplaats, query: &str, limit: u32) -> Result {
    for listing in marktplaats.search(query, limit).await?.listings {
        info!(
            id = listing.item_id,
            timestamp = %listing.timestamp,
            title = listing.title,
            n_pictures = listing.pictures.len(),
            n_image_urls = listing.image_urls.len(),
            price = ?listing.price,
            seller_name = listing.seller.name,
            "🌠",
        );
    }
    Ok(())
}

#[instrument(skip_all)]
async fn get_updates(telegram: &Telegram, request: GetUpdates) -> Result {
    for update in telegram.call(request).await? {
        info!(update.id, ?update.payload);
    }
    Ok(())
}
