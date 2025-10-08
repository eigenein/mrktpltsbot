use std::path::PathBuf;

use clap::{Parser, Subcommand};
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about, propagate_version = true)]
pub struct Args {
    /// Sentry DSN: <https://docs.sentry.io/concepts/key-terms/dsn-explainer/>.
    #[clap(long, env = "SENTRY_DSN", hide_env_values = true)]
    pub sentry_dsn: Option<String>,

    /// Pydantic Logfire token: <https://logfire.pydantic.dev/docs/how-to-guides/create-write-tokens/>.
    #[clap(long, env = "LOGFIRE_TOKEN", hide_env_values = true)]
    _logfire_token: Option<String>,

    /// SQLite database path.
    #[expect(clippy::doc_markdown)]
    #[clap(long, env = "DB", default_value = "mrktpltsbot.sqlite3", hide_env_values = true)]
    pub db: PathBuf,

    /// Enable tracing of HTTP requests for debugging.
    #[clap(long = "trace-requests", env = "TRACE_REQUESTS", hide_env_values = true)]
    pub trace_requests: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the bot indefinitely.
    Run(Box<RunArgs>),
}

#[derive(Parser)]
pub struct RunArgs {
    /// Search interval, in seconds.
    #[clap(
        long = "search-interval-secs",
        env = "SEARCH_INTERVAL_SECS",
        default_value = "60",
        hide_env_values = true
    )]
    pub search_interval_secs: u64,

    #[command(flatten)]
    pub telegram: TelegramArgs,

    #[command(flatten)]
    pub marktplaats: MarktplaatsArgs,
}

#[derive(Parser)]
#[clap(next_help_heading = "Marktplaats")]
pub struct MarktplaatsArgs {
    /// Limit of Marktplaats search results per query.
    #[clap(
        long = "marktplaats-search-limit",
        env = "MARKTPLAATS_SEARCH_LIMIT",
        default_value = "10",
        hide_env_values = true
    )]
    pub marktplaats_search_limit: u32,

    /// Heartbeat URL for the Marktplaats connection.
    #[clap(
        long = "marktplaats-heartbeat-url",
        env = "MARKTPLAATS_HEARTBEAT_URL",
        id = "marktplaats_heartbeat_url",
        hide_env_values = true
    )]
    pub heartbeat_url: Option<Url>,

    /// Enable search in descriptions for Marktplaats.
    #[clap(
        long = "marktplaats-search-in-title-and-description",
        env = "MARKTPLAATS_SEARCH_IN_TITLE_AND_DESCRIPTION",
        hide_env_values = true
    )]
    pub search_in_title_and_description: bool,
}

#[derive(Parser)]
#[clap(next_help_heading = "Telegram")]
pub struct TelegramArgs {
    /// Telegram bot token: <https://core.telegram.org/bots/api#authorizing-your-bot>.
    #[clap(long = "telegram-bot-token", env = "TELEGRAM_BOT_TOKEN", hide_env_values = true)]
    pub bot_token: String,

    /// Timeout for Telegram long polling, in seconds.
    #[clap(
        long = "telegram-poll-timeout-secs",
        env = "TELEGRAM_POLL_TIMEOUT_SECS",
        default_value = "60",
        hide_env_values = true
    )]
    pub poll_timeout_secs: u64,

    /// Authorize chat ID to use the bot.
    #[clap(
        long = "telegram-authorize-chat-id",
        env = "TELEGRAM_AUTHORIZED_CHAT_IDS",
        value_delimiter = ',',
        visible_alias = "chat-id",
        hide_env_values = true
    )]
    pub authorized_chat_ids: Vec<i64>,

    /// Heartbeat URL for the Telegram bot.
    #[clap(
        long = "telegram-heartbeat-url",
        env = "TELEGRAM_HEARTBEAT_URL",
        id = "telegram_heartbeat_url",
        hide_env_values = true
    )]
    pub heartbeat_url: Option<Url>,
}
