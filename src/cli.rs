use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about, propagate_version = true)]
pub struct Args {
    /// Sentry DSN: <https://docs.sentry.io/concepts/key-terms/dsn-explainer/>.
    #[clap(long, env = "SENTRY_DSN")]
    pub sentry_dsn: Option<String>,

    /// SQLite database path.
    #[expect(clippy::doc_markdown)]
    #[clap(long, env = "DB", default_value = "mrktpltsbot.sqlite3")]
    pub db: PathBuf,

    /// Crawling interval, in seconds.
    #[clap(long, env = "MARKTPLAATS_CRAWL_INTERVAL_SECS", default_value = "60")]
    pub marktplaats_crawl_interval_secs: u64,

    #[command(flatten)]
    pub telegram: TelegramArgs,
}

#[derive(Parser)]
pub struct TelegramArgs {
    /// Telegram bot token: <https://core.telegram.org/bots/api#authorizing-your-bot>.
    #[clap(long, env = "TELEGRAM_BOT_TOKEN")]
    pub bot_token: String,

    /// Timeout for Telegram long polling, in seconds.
    #[clap(long, env = "TELEGRAM_POLL_TIMEOUT_SECS", default_value = "60")]
    pub poll_timeout_secs: u64,

    /// Authorize chat ID to use the bot.
    #[clap(
        long = "authorize-chat-id",
        env = "TELEGRAM_AUTHORIZED_CHAT_IDS",
        value_delimiter = ',',
        alias = "chat-id"
    )]
    pub authorized_chat_ids: Vec<i64>,
}
