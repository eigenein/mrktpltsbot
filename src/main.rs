use std::time::Duration;

use clap::Parser;
use tokio::time::sleep;

use crate::{
    cli::{Cli, Command},
    client::build_client,
    marktplaats::{listing::Listings, Marktplaats},
    prelude::*,
    telegram::{
        error::TelegramError,
        methods::{GetMe, GetUpdates, ParseMode, SendMessage},
        objects::ChatId,
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
    let marktplaats = Marktplaats(client.clone());

    match cli.command {
        Command::Run(args) => {
            unimplemented!()
        }

        Command::QuickSearch { query, limit } => {
            let listings: Listings =
                serde_json::from_str(&marktplaats.search(&query, limit).await?)?;
            for listing in listings {
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

        Command::GetMe { bot_token } => {
            let user = Telegram::new(client, bot_token.into()).call(GetMe).await?;
            info!(user.id, user.username);
            Ok(())
        }

        Command::GetUpdates(args) => {
            let request = GetUpdates {
                offset: args.offset,
                limit: args.limit,
                timeout_secs: args.timeout_secs,
                allowed_updates: args.allowed_updates,
            };
            let updates = Telegram::new(client, args.bot_token.into())
                .call(request)
                .await?;
            info!(n_updates = updates.len());
            for update in updates {
                info!(update.id, ?update.payload);
            }
            Ok(())
        }

        Command::SendMessage(args) => {
            let telegram = Telegram::new(client, args.bot_token.into());
            for _ in 0..args.repeat {
                loop {
                    let request = SendMessage {
                        chat_id: ChatId::Integer(args.chat_id),
                        parse_mode: Some(ParseMode::Html),
                        text: args.html.clone(),
                    };
                    match telegram.call(request).await {
                        Ok(message) => {
                            info!(message.id);
                            break;
                        }
                        Err(TelegramError::TooManyRequests { retry_after, .. }) => {
                            warn!(retry_after.secs, "Too many requests");
                            sleep(Duration::from_secs(retry_after.secs)).await;
                        }
                        Err(error) => {
                            bail!("{error:#}")
                        }
                    }
                }
            }
            Ok(())
        }
    }
}
