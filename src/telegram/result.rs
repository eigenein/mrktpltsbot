use anyhow::anyhow;

use crate::telegram::{error::TelegramError, Response};

pub type TelegramResult<T> = anyhow::Result<T, TelegramError>;

impl<T> From<Response<T>> for TelegramResult<T> {
    fn from(response: Response<T>) -> Self {
        match response {
            Response::Ok { result, .. } => Ok(result),

            Response::Err {
                error_code: 429,
                parameters,
                ..
            } => Err(TelegramError::TooManyRequests(
                parameters
                    .and_then(|parameters| parameters.retry_after_secs)
                    .unwrap_or_default(),
            )),

            Response::Err {
                description,
                error_code,
                ..
            } => Err(TelegramError::Other(anyhow!("#{error_code} {description}"))),
        }
    }
}
