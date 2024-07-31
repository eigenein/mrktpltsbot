use clap::{Parser, Subcommand};

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
    /// Perform manual Marktplaats search.
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
}
