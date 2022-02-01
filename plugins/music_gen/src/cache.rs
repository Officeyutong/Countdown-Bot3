use std::sync::Arc;

use countdown_bot3::countdown_bot::client::ResultType;
use redis::AsyncCommands;
fn make_key(hash: &str) -> String {
    format!("countdownbot-music-{}", hash)
}
pub async fn check_from_cache(
    client: Arc<redis::Client>,
    hash: &str,
) -> ResultType<Option<Vec<u8>>> {
    let mut conn = client.get_async_connection().await?;
    let key = make_key(hash);
    if conn.exists(&key).await? {
        let output: Vec<u8> = conn.get(&key).await?;
        return Ok(Some(output));
    } else {
        return Ok(None);
    }
}

pub async fn store_into_cache(
    client: Arc<redis::Client>,
    hash: &str,
    bytes: &[u8],
    timeout: usize,
) -> ResultType<()> {
    let key = make_key(hash);
    let mut conn = client.get_async_connection().await?;
    if conn.exists(&key).await? {
        return Ok(());
    }
    conn.set_ex(&key, bytes, timeout).await?;
    return Ok(());
}
