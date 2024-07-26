use crate::{prelude::*, redis::pop_notification, telegram::Telegram};

const ONE_SECOND: Duration = Duration::from_secs(1);

#[must_use]
pub struct Notifier {
    redis: RedisConnection,
    telegram: Telegram,
}

impl Notifier {
    pub fn new(redis: RedisConnection, telegram: Telegram) -> Self {
        Self { redis, telegram }
    }

    pub async fn run(mut self) -> Result {
        info!("Running…");
        loop {
            let notification = pop_notification(&mut self.redis).await?;
            info!("Notification to the chat #{}.", notification.chat_id);

            #[allow(clippy::if_not_else)]
            if !notification.image_urls.is_empty() {
                if notification.image_urls.len() == 1 {
                    self.telegram
                        .send_photo(
                            notification.chat_id,
                            &notification.image_urls[0],
                            &notification.text,
                            notification.reply_markup,
                        )
                        .await
                        .log_result();
                } else {
                    self.telegram
                        .send_media_group(
                            notification.chat_id,
                            &notification.text,
                            notification.image_urls,
                        )
                        .await
                        .log_result();
                }
            } else {
                self.telegram
                    .send_message(
                        notification.chat_id,
                        &notification.text,
                        notification.reply_markup,
                    )
                    .await
                    .log_result();
            }

            // https://core.telegram.org/bots/faq#my-bot-is-hitting-limits-how-do-i-avoid-this
            task::sleep(ONE_SECOND).await;
        }
    }
}
