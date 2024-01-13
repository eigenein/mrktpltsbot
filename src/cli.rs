use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[clap(long, env = "SENTRY_DSN")]
    pub sentry_dsn: Option<String>,

    #[clap(long, env = "SENTRY_TRACES_SAMPLE_RATE", default_value = "1.0")]
    pub traces_sample_rate: f32,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the crawler.
    Crawler,
}
