//! Telegram bot [API].
//!
//! [API]: https://core.telegram.org/bots/api

use serde::de::DeserializeOwned;

use crate::{prelude::*, telegram::types::*};

pub mod format;
pub mod notifier;
pub mod reply_markup;
pub mod types;

pub const MARKDOWN_V2: Option<&str> = Some("MarkdownV2");

const GET_UPDATES_TIMEOUT: u64 = 60;
const GET_UPDATES_REQUEST_TIMEOUT: Duration = Duration::from_secs(GET_UPDATES_TIMEOUT + 5);

/// <https://core.telegram.org/bots/api>
#[must_use]
pub struct Telegram {
    /// <https://core.telegram.org/bots#6-botfather>
    base_url: String,
}

impl Telegram {
    pub fn new(token: &str) -> Self {
        Self {
            base_url: format!("https://api.telegram.org/bot{token}"),
        }
    }

    /// <https://core.telegram.org/bots/api#setmycommands>
    pub async fn set_my_commands(&self, commands: Vec<BotCommand>) -> Result<bool> {
        self.call("setMyCommands", &json!({ "commands": commands }), None).await
    }

    /// <https://core.telegram.org/bots/api#getupdates>
    pub async fn get_updates(&self, offset: i64, allowed_updates: &[&str]) -> Result<Vec<Update>> {
        self.call(
            "getUpdates",
            &json!({
                "offset": offset,
                "allowed_updates": allowed_updates,
                "timeout": GET_UPDATES_TIMEOUT,
            }),
            Some(GET_UPDATES_REQUEST_TIMEOUT),
        )
        .await
    }

    /// <https://core.telegram.org/bots/api#sendmessage>
    pub async fn send_message<RM: Into<Option<ReplyMarkup>>>(
        &self,
        chat_id: i64,
        text: &str,
        parse_mode: Option<&str>,
        reply_markup: RM,
    ) -> Result<Message> {
        self.call(
            "sendMessage",
            &json!({
                "chat_id": chat_id,
                "text": text,
                "parse_mode": parse_mode,
                "reply_markup": serialize_reply_markup(&reply_markup.into())?,
            }),
            None,
        )
        .await
    }

    /// <https://core.telegram.org/bots/api#sendphoto>
    pub async fn send_photo<RM: Into<Option<ReplyMarkup>>>(
        &self,
        chat_id: i64,
        photo: &str,
        caption: Option<&str>,
        parse_mode: Option<&str>,
        reply_markup: RM,
    ) -> Result<Message> {
        self.call(
            "sendPhoto",
            &json!({
                "chat_id": chat_id,
                "photo": photo,
                "caption": caption,
                "parse_mode": parse_mode,
                "reply_markup": serialize_reply_markup(&reply_markup.into())?,
            }),
            None,
        )
        .await
    }

    pub async fn answer_callback_query(&self, callback_query_id: &str) -> Result<bool> {
        self.call("answerCallbackQuery", &json!({ "callback_query_id": callback_query_id }), None)
            .await
    }

    /// Call the Bot API method.
    async fn call<A: Serialize, R: DeserializeOwned>(
        &self,
        method_name: &str,
        args: &A,
        timeout: Option<Duration>,
    ) -> Result<R> {
        info!("{}…", method_name);
        retry_notify(
            ExponentialBackoff::default(),
            || async {
                let mut request_builder =
                    CLIENT.get(format!("{}/{}", self.base_url, method_name)).json(&args);
                if let Some(timeout) = timeout {
                    request_builder = request_builder.timeout(timeout);
                }
                let response =
                    request_builder.send().await.context("failed to send the request")?;
                if !response.status().is_success() {
                    return Err(backoff::Error::from(anyhow!(
                        "HTTP {}: `{}`",
                        response.status(),
                        response.text().await.as_deref().unwrap_or("<failed to read>")
                    )));
                }
                Ok(response
                    .json::<TelegramResult<R>>()
                    .await
                    .context("failed to parse the response")?
                    .result)
            },
            |error, _| log_error(anyhow!("{}: {}", method_name, error)),
        )
        .await
    }
}

fn serialize_reply_markup(reply_markup: &Option<ReplyMarkup>) -> Result<String> {
    Ok(if let Some(reply_markup) = reply_markup {
        serde_json::to_string(reply_markup)?
    } else {
        "{}".into()
    })
}
