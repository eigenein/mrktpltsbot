use std::path::PathBuf;

use clap::Parser;

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

    /// Timeout  for long polling, in seconds.
    #[clap(long, env = "TIMEOUT_SECS", default_value = "60")]
    pub timeout_secs: u64,

    /// Crawling interval, in seconds.
    #[clap(long, env = "CRAWL_INTERVAL_SECS", default_value = "60")]
    pub crawl_interval_secs: u64,

    /// Authorize chat ID to use the bot.
    #[clap(long = "authorize-chat-id", alias = "chat-id")]
    pub authorized_chat_ids: Vec<i64>,
}
