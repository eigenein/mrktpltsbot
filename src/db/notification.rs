use sqlx::SqliteConnection;

use crate::prelude::*;

/// Proof that the chat has received the item notification.
#[derive(Eq, PartialEq)]
pub struct Notification {
    pub item_id: String,
    pub chat_id: i64,
}

pub struct Notifications<'a>(pub &'a mut SqliteConnection);

impl Notifications<'_> {
    #[instrument(
        name = "💾 Upserting notification…",
        level = Level::DEBUG,
        skip_all,
        fields(item_id = &notification.item_id, chat_id = notification.chat_id),
    )]
    pub async fn upsert(&mut self, notification: &Notification) -> Result {
        sqlx::query(
            // language=sql
            "INSERT INTO notifications (item_id, chat_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
        )
        .bind(&notification.item_id)
        .bind(notification.chat_id)
        .execute(&mut *self.0)
        .await
        .context("failed to upsert the notification")?;

        Ok(())
    }

    #[instrument(
        name = "💾 Checking notification…",
        level = Level::DEBUG,
        skip_all,
        fields(item_id = &notification.item_id, chat_id = notification.chat_id),
    )]
    pub async fn exists(&mut self, notification: &Notification) -> Result<bool> {
        // language=sql
        const QUERY: &str =
            "SELECT EXISTS(SELECT 1 FROM notifications WHERE item_id = ?1 AND chat_id = ?2)";
        sqlx::query_scalar(QUERY)
            .bind(&notification.item_id)
            .bind(notification.chat_id)
            .fetch_one(&mut *self.0)
            .await
            .context("failed to check for existence of notification")
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use chrono::Utc;

    use super::*;
    use crate::db::{
        Db,
        item::{Item, Items},
    };

    #[tokio::test]
    async fn test_exists_ok() -> Result {
        let db = Db::try_new(Path::new(":memory:")).await?;
        let mut connection = db.connection().await;

        let item = Item { id: "m42", updated_at: Utc::now() };
        Items(&mut connection).upsert(item).await?;

        let notification_1 = Notification { item_id: "m42".to_string(), chat_id: 42 };

        let mut notifications = Notifications(&mut connection);

        notifications.upsert(&notification_1).await?;
        assert!(notifications.exists(&notification_1).await?);

        let notification_2 = Notification { item_id: "m42".to_string(), chat_id: 43 };
        assert!(!notifications.exists(&notification_2).await?);

        Ok(())
    }
}
