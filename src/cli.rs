use std::path::PathBuf;

use clap::Parser;
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about, propagate_version = true)]
pub struct Args {
    /// Sentry DSN: <https://docs.sentry.io/concepts/key-terms/dsn-explainer/>.
    #[clap(long, env = "SENTRY_DSN", hide_env_values = true)]
    pub sentry_dsn: Option<String>,

    /// SQLite database path.
    #[expect(clippy::doc_markdown)]
    #[clap(long, env = "DB", default_value = "mrktpltsbot.sqlite3", hide_env_values = true)]
    pub db: PathBuf,

    #[command(flatten)]
    pub telegram: TelegramArgs,

    #[command(flatten)]
    pub marktplaats: MarktplaatsArgs,
}

#[derive(Parser)]
#[clap(next_help_heading = "Marktplaats")]
pub struct MarktplaatsArgs {
    /// Crawling interval, in seconds.
    #[clap(
        long = "marktplaats-crawl-interval-secs",
        env = "MARKTPLAATS_CRAWL_INTERVAL_SECS",
        default_value = "60",
        hide_env_values = true
    )]
    pub crawl_interval_secs: u64,

    /// Limit of Marktplaats search results per query.
    #[clap(
        long = "marktplaats-search-limit",
        env = "MARKTPLAATS_SEARCH_LIMIT",
        default_value = "30",
        hide_env_values = true
    )]
    pub search_limit: u32,

    /// Better Stack heartbeat URL for the Marktplaats crawler.
    #[clap(
        long = "marktplaats-heartbeat-url",
        env = "MARKTPLAATS_HEARTBEAT_URL",
        id = "marktplaats_heartbeat_url",
        hide_env_values = true
    )]
    pub heartbeat_url: Option<Url>,
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
        alias = "chat-id",
        hide_env_values = true
    )]
    pub authorized_chat_ids: Vec<i64>,

    /// Better Stack heartbeat URL for the Telegram bot.
    #[clap(
        long = "telegram-heartbeat-url",
        env = "TELEGRAM_HEARTBEAT_URL",
        id = "telegram_heartbeat_url",
        hide_env_values = true
    )]
    pub heartbeat_url: Option<Url>,
}
