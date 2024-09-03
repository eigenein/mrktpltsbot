#[derive(Debug, thiserror::Error)]
pub enum TelegramError {
    #[error("too many requests, retry after {0} secs")]
    TooManyRequests(u32),

    #[error("other error")]
    Other(#[from] anyhow::Error),
}
