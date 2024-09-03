use anyhow::anyhow;

use crate::telegram::{error::TelegramError, Response};

pub type TelegramResult<T> = anyhow::Result<T, TelegramError>;

impl<T> From<Response<T>> for TelegramResult<T> {
    fn from(response: Response<T>) -> Self {
        match response {
            Response::Ok { result, .. } => Ok(result),

            Response::TooManyRequests { retry_after, .. } => {
                Err(TelegramError::TooManyRequests(retry_after.secs))
            }

            Response::OtherError {
                description,
                error_code,
                ..
            } => Err(TelegramError::Other(anyhow!("#{error_code} {description}"))),
        }
    }
}
