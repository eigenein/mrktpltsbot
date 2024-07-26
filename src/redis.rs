//! Redis extensions.

use redis::{
    Client, ConnectionAddr, ConnectionInfo, FromRedisValue, RedisConnectionInfo, ToRedisArgs,
    aio::ConnectionLike,
};

use crate::{prelude::*, telegram::types::ReplyMarkup};

/// An auto-incrementing subscription ID.
const SUBSCRIPTION_COUNTER_KEY: &str = "subscriptions::counter";

/// A set of subscription IDs.
const ALL_SUBSCRIPTIONS_KEY: &str = "subscriptions::all";

/// Notification queue.
const NOTIFICATIONS_KEY: &str = "notifications";

/// Ad "seen" flag expiration time.
const SEEN_TTL_SECS: u64 = 30 * 24 * 60 * 60;

// TODO: consider storing `method_name` and `args` and calling `telegram.call` directly.
#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub chat_id: i64,
    pub text: String,
    pub reply_markup: Option<ReplyMarkup>,
    pub image_urls: Vec<String>,
}

/// Open the Redis connection.
pub fn open(db: i64) -> Result<Client> {
    // TODO: wrap the Redis connection into a struct.
    info!("Connecting to Redis #{}…", db);
    Ok(Client::open(ConnectionInfo {
        addr: ConnectionAddr::Tcp("localhost".into(), 6379),
        redis: RedisConnectionInfo { db, username: None, password: None },
    })?)
}

/// Store the subscription in the database.
pub async fn subscribe_to<C: AsyncCommands>(
    connection: &mut C,
    chat_id: i64,
    query: &str,
) -> Result<(i64, i64)> {
    // TODO: return a struct.
    let subscription_id = new_subscription_id(connection).await?;
    info!("New subscription #{}.", subscription_id);
    connection
        .hset_multiple(get_subscription_details_key(subscription_id), &[
            ("chat_id", chat_id.to_string().as_str()),
            ("query", query),
        ])
        .await?;
    connection.sadd(ALL_SUBSCRIPTIONS_KEY, subscription_id).await?;
    let subscription_count = connection.scard(ALL_SUBSCRIPTIONS_KEY).await?;
    Ok((subscription_id, subscription_count))
}

/// Delete the subscription from the database.
pub async fn unsubscribe_from<C: AsyncCommands>(
    connection: &mut C,
    subscription_id: i64,
) -> Result<i64> {
    connection.srem(ALL_SUBSCRIPTIONS_KEY, subscription_id).await?;
    connection.del(get_subscription_details_key(subscription_id)).await?;
    Ok(connection.scard(ALL_SUBSCRIPTIONS_KEY).await?)
}

/// Returns all subscription IDs.
pub async fn list_subscriptions<C: AsyncCommands>(connection: &mut C) -> Result<Vec<i64>> {
    Ok(connection.smembers(ALL_SUBSCRIPTIONS_KEY).await?)
}

/// Picks a random subscription and returns the related chat ID and query.
pub async fn pick_random_subscription<C>(connection: &mut C) -> Result<Option<(i64, i64, String)>>
where
    C: AsyncCommands,
{
    let subscription_id: Option<i64> = connection.srandmember(ALL_SUBSCRIPTIONS_KEY).await?;
    info!("Picked subscription `{:?}`.", subscription_id);
    if let Some(subscription_id) = subscription_id {
        let (chat_id, query) = get_subscription_details(connection, subscription_id).await?;
        Ok(Some((subscription_id, chat_id, query)))
    } else {
        Ok(None)
    }
}

/// Marks the item as seen. Returns whether it has been seen for the first time.
pub async fn check_seen<C: AsyncCommands>(
    connection: &mut C,
    chat_id: i64,
    item_id: &str,
) -> Result<bool> {
    set_nx_ex(connection, &get_seen_key(chat_id, item_id), 1, SEEN_TTL_SECS).await
}

/// Wait for a notification in the queue.
pub async fn pop_notification<C: AsyncCommands>(connection: &mut C) -> Result<Notification> {
    let (_, notification): (String, String) = connection.blpop(NOTIFICATIONS_KEY, 0.0).await?;
    Ok(serde_json::from_str(&notification)?)
}

/// Push the notification to the queue.
pub async fn push_notification<C: AsyncCommands>(
    connection: &mut C,
    notification: Notification,
) -> Result {
    Ok(connection
        .rpush(NOTIFICATIONS_KEY, serde_json::to_string(&notification)?)
        .await?)
}

/// Returns the related chat ID and query by subscription ID.
pub async fn get_subscription_details<C: AsyncCommands>(
    connection: &mut C,
    subscription_id: i64,
) -> Result<(i64, String)> {
    Ok(redis::cmd("HMGET")
        .arg(get_subscription_details_key(subscription_id))
        .arg("chat_id")
        .arg("query")
        .query_async(connection)
        .await?)
}

/// Set the value if not exists with the expiry time.
async fn set_nx_ex<C, V, R>(connection: &mut C, key: &str, value: V, expiry_time: u64) -> Result<R>
where
    C: ConnectionLike,
    V: ToRedisArgs,
    R: FromRedisValue,
{
    Ok(redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("NX")
        .arg("EX")
        .arg(expiry_time)
        .query_async(connection)
        .await?)
}

/// Reserve and return a new subscription ID.
async fn new_subscription_id<C: AsyncCommands>(connection: &mut C) -> Result<i64> {
    Ok(connection.incr(SUBSCRIPTION_COUNTER_KEY, 1).await?)
}

fn get_subscription_details_key(subscription_id: i64) -> String {
    format!("subscriptions::{subscription_id}")
}

fn get_seen_key(chat_id: i64, item_id: &str) -> String {
    format!("items::{chat_id}::seen::{item_id}")
}
