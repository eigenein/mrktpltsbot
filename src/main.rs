use clap::Parser;

use crate::{
    cli::{Cli, Command},
    client::build_client,
    marktplaats::{listing::Listings, Marktplaats, SearchRequest, SortBy, SortOrder},
    prelude::*,
    telegram::{
        listing::SendListingRequest,
        methods::{GetMe, GetUpdates, SendMessage},
        objects::{ChatId, ParseMode},
        Telegram,
    },
};

mod cli;
mod client;
mod db;
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
    let telegram = Telegram::new(client, cli.bot_token);

    match cli.command {
        Command::Run(_args) => {
            unimplemented!()
        }

        Command::QuickSearch {
            query,
            limit,
            chat_id,
        } => {
            let request = SearchRequest::builder()
                .query(&query)
                .limit(limit)
                .sort_by(SortBy::SortIndex)
                .sort_order(SortOrder::Decreasing)
                .search_in_title_and_description(true)
                .build();
            let listings: Listings = serde_json::from_str(&marktplaats.search(&request).await?)?;
            for listing in listings {
                info!(?listing, "Found advertisement");
                if let Some(chat_id) = chat_id {
                    match SendListingRequest::build(ChatId::Integer(chat_id), &listing) {
                        SendListingRequest::Message(request) => {
                            let message = telegram.call(request).await?;
                            info!(?message, "Sent");
                        }
                        SendListingRequest::Photo(request) => {
                            let message = telegram.call(request).await?;
                            info!(?message, "Sent");
                        }
                        SendListingRequest::MediaGroup(request) => {
                            let messages = telegram.call(request).await?;
                            info!(?messages, "Sent");
                        }
                    };
                }
            }
            Ok(())
        }

        Command::GetMe => {
            let user = telegram.call(GetMe).await?;
            info!(user.id, user.username, "I am");
            Ok(())
        }

        Command::GetUpdates(args) => {
            let request = GetUpdates {
                offset: args.offset,
                limit: args.limit,
                timeout_secs: args.timeout_secs,
                allowed_updates: args.allowed_updates,
            };
            let updates = telegram.call(request).await?;
            info!(n_updates = updates.len());
            for update in updates {
                info!(update.id, ?update.payload, "Received");
            }
            Ok(())
        }

        Command::SendMessage(args) => {
            for _ in 0..args.repeat {
                let request = SendMessage::builder()
                    .chat_id(ChatId::Integer(args.chat_id))
                    .text(&args.html)
                    .parse_mode(ParseMode::Html)
                    .build();
                let message = telegram.call(request).await?;
                info!(?message, "Sent");
            }
            Ok(())
        }
    }
}
