use clap::Parser;

use crate::{
    bot::Bot,
    cli::{Cli, Command},
    client::build_client,
    db::Db,
    marktplaats::{Marktplaats, SearchRequest, SortBy, SortOrder},
    prelude::*,
    telegram::{
        listing::ListingView,
        methods::{GetMe, GetUpdates, Method, SendMessage},
        objects::{ChatId, ParseMode},
        Telegram,
    },
};

mod bot;
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
    let db = Db::new(&cli.db).await?;
    let client = build_client()?;
    let marktplaats = Marktplaats(client.clone());
    let telegram = Telegram::new(client, cli.bot_token);

    match cli.command {
        Command::Run(args) => Bot::builder()
            .telegram(telegram)
            .marktplaats(marktplaats)
            .db(db)
            .timeout_secs(args.timeout_secs)
            .build()
            .run_telegram()
            .await
            .context("fatal error"),

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

            let listings = marktplaats.search(&request).await?;
            info!(n_listings = listings.inner.len());

            for listing in listings {
                info!(listing.item_id, listing.title, "Found advertisement");
                if let Some(chat_id) = chat_id {
                    ListingView::with(ChatId::Integer(chat_id), &listing)
                        .call_on(&telegram)
                        .await?;
                }
            }

            Ok(())
        }

        Command::GetMe => {
            let user = telegram.call(&GetMe).await?;
            info!(user.id, user.username, "I am");
            Ok(())
        }

        Command::GetUpdates(args) => {
            let updates = GetUpdates {
                offset: args.offset,
                limit: args.limit,
                timeout_secs: args.timeout_secs,
                allowed_updates: args.allowed_updates.as_deref(),
            }
            .call_on(&telegram)
            .await?;
            info!(n_updates = updates.len());

            for update in updates {
                info!(update.id, ?update.payload, "Received");
            }

            Ok(())
        }

        Command::SendMessage(args) => {
            let request = SendMessage::builder()
                .chat_id(ChatId::Integer(args.chat_id))
                .text(&args.html)
                .parse_mode(ParseMode::Html)
                .build();
            for _ in 0..args.repeat {
                let message = request.call_on(&telegram).await?;
                info!(?message, "Sent");
            }
            Ok(())
        }
    }
}
