use clap::{Parser, Subcommand};

use crate::telegram::requests::AllowedUpdate;

#[derive(Parser)]
#[command(author, version, about, long_about, propagate_version = true)]
pub struct Cli {
    #[clap(long, env = "SENTRY_DSN")]
    pub sentry_dsn: Option<String>,

    #[clap(long, env = "BOT_TOKEN")]
    pub bot_token: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manually search Marktplaats.
    #[clap(alias = "search")]
    QuickSearch {
        /// Search query.
        query: String,

        /// Maximum number of results.
        #[clap(long, default_value = "1")]
        limit: u32,
    },

    /// Test Telegram bot API token.
    GetMe,

    /// Manually check out the bot updates.
    GetUpdates {
        #[clap(long)]
        offset: Option<u32>,

        #[clap(long)]
        limit: Option<u32>,

        #[clap(long)]
        timeout_secs: Option<u64>,

        #[clap(long, value_delimiter = ',', num_args = 1..)]
        allowed_updates: Option<Vec<AllowedUpdate>>,
    },
}
