use clap::Parser;
use maud::Render;

use crate::{
    cli::{Cli, Command},
    client::build_client,
    marktplaats::{listing::Listings, Marktplaats, SearchRequest, SortBy, SortOrder},
    prelude::*,
    telegram::{
        methods::{GetMe, GetUpdates, SendMessage},
        objects::{
            ChatId,
            InlineKeyboardButton,
            InlineKeyboardButtonPayload,
            InlineKeyboardMarkup,
            LinkPreviewOptions,
            ParseMode,
        },
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
    let telegram = Telegram::new(client, cli.bot_token);

    match cli.command {
        Command::Run(args) => {
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
                info!(
                    id = listing.item_id,
                    timestamp = %listing.timestamp,
                    title = listing.title,
                    n_pictures = listing.pictures.len(),
                    n_image_urls = listing.image_urls.len(),
                    price = ?listing.price,
                    seller_name = listing.seller.name,
                    "Found advertisement",
                );
                if let Some(chat_id) = chat_id {
                    let html = listing.render().into_string();
                    let url = listing.https_url();
                    let request = SendMessage::builder()
                        .chat_id(ChatId::Integer(chat_id))
                        .text(&html)
                        .parse_mode(ParseMode::Html)
                        .link_preview_options(
                            LinkPreviewOptions::builder().is_disabled(true).build(),
                        )
                        .reply_markup(InlineKeyboardMarkup::single_button(InlineKeyboardButton {
                            text: "View",
                            payload: InlineKeyboardButtonPayload::Url(&url),
                        }))
                        .build();
                    let message = telegram.call(request).await?;
                    info!(message = ?message, "Sent");
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
                info!(message.id);
            }
            Ok(())
        }
    }
}
