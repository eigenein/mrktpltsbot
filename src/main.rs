use clap::Parser;

use crate::{
    cli::{Cli, Command},
    client::build_client,
    marktplaats::Marktplaats,
    prelude::*,
};

mod cli;
mod client;
mod marktplaats;
mod math;
mod prelude;
mod tracing;

#[tokio::main]
async fn main() -> Result {
    let cli = Cli::parse();
    let _tracing_guards = tracing::init(cli.sentry_dsn.as_deref())?;
    fallible_main(cli)
        .await
        .inspect_err(|error| error!("Fatal error: {error:#}"))
}

async fn fallible_main(cli: Cli) -> Result {
    let client = build_client()?;
    let marktplaats = Marktplaats(client);

    match cli.command {
        Command::Search { query, limit } => {
            for listing in marktplaats.search(&query, limit).await?.listings {
                info!(
                    id = listing.item_id,
                    timestamp = %listing.timestamp,
                    title = listing.title,
                    n_pictures = listing.pictures.len(),
                    n_image_urls = listing.image_urls.len(),
                    cents = listing.price.cents,
                    price_type = ?listing.price.type_,
                    "🌠",
                );
            }
            Ok(())
        }
    }
}
