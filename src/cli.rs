use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::telegram::methods::AllowedUpdate;

#[derive(Parser)]
#[command(author, version, about, long_about, propagate_version = true)]
pub struct Cli {
    /// Sentry DSN: <https://docs.sentry.io/concepts/key-terms/dsn-explainer/>.
    #[clap(long, env = "SENTRY_DSN")]
    pub sentry_dsn: Option<String>,

    /// Telegram bot token: <https://core.telegram.org/bots/api#authorizing-your-bot>.
    #[clap(long, env = "BOT_TOKEN")]
    pub bot_token: String,

    /// SQLite database path.
    #[expect(clippy::doc_markdown)]
    #[clap(long, env = "DB", default_value = "mrktpltsbot.sqlite3")]
    pub db: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Parser)]
pub struct GetUpdates {
    #[clap(long)]
    pub offset: Option<u64>,

    #[clap(long)]
    pub limit: Option<u32>,

    #[clap(long)]
    pub timeout_secs: Option<u64>,

    #[clap(long, value_delimiter = ',', num_args = 1..)]
    pub allowed_updates: Option<Vec<AllowedUpdate>>,
}

#[derive(Parser)]
pub struct SendMessage {
    /// Send the same message this many times.
    #[clap(long, default_value = "1")]
    pub repeat: usize,

    #[clap(long)]
    pub chat_id: i64,

    #[clap()]
    pub html: String,
}

#[derive(Parser)]
pub struct Run {
    /// Timeout in seconds for long polling.
    #[clap(long = "timeout", env = "TIMEOUT", default_value = "60")]
    pub timeout_secs: u64,
}

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

        /// Send the listings to the specified chat.
        #[clap(long)]
        chat_id: Option<i64>,
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
