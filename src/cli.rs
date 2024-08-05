use clap::{Parser, Subcommand};

use crate::telegram::methods::AllowedUpdate;

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

#[derive(Parser)]
pub struct GetUpdates {
    #[clap(long)]
    pub offset: Option<u32>,

    #[clap(long)]
    pub limit: Option<u32>,

    #[clap(long)]
    pub timeout_secs: Option<u64>,

    #[clap(long, value_delimiter = ',', num_args = 1..)]
    pub allowed_updates: Option<Vec<AllowedUpdate>>,
}

#[derive(Parser)]
pub struct SendMessage {
    #[clap(long)]
    pub chat_id: i64,

    #[clap()]
    pub html: String,
}

#[derive(Parser)]
pub struct Run {}

#[derive(Subcommand)]
pub enum Command {
    /// Run the bot indefinitely.
    Run(Run),

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
    #[clap(alias = "me")]
    GetMe,

    /// Manually check out the bot updates.
    #[clap(alias = "updates")]
    GetUpdates(GetUpdates),

    /// Send out a test message.
    #[clap(alias = "message")]
    SendMessage(SendMessage),
}
