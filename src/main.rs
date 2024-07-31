use clap::Parser;

use crate::{
    cli::{Cli, Command},
    client::build_client,
    marktplaats::Marktplaats,
    prelude::*,
};

mod cli;
mod client;
mod logging;
mod marktplaats;
mod math;
mod prelude;

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
    let marktplaats = Marktplaats(client);

    match cli.command {
        Command::QuickSearch { query, limit } => quick_search(&marktplaats, &query, limit).await,
    }
}

#[instrument(skip_all)]
async fn quick_search(marktplaats: &Marktplaats, query: &str, limit: u32) -> Result {
    for listing in marktplaats.search(&query, limit).await?.listings {
        info!(
            id = listing.item_id,
            timestamp = %listing.timestamp,
            title = listing.title,
            n_pictures = listing.pictures.len(),
            n_image_urls = listing.image_urls.len(),
            cents = listing.price.cents,
            price_type = ?listing.price.type_,
            seller_name = listing.seller.name,
            "🌠",
        );
    }
    Ok(())
}
