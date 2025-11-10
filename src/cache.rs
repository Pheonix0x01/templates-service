use redis::aio::ConnectionManager;
use redis::{Client, RedisError};

pub type RedisPool = ConnectionManager;

pub async fn create_redis_pool(redis_url: &str) -> Result<RedisPool, RedisError> {
    let client = Client::open(redis_url)?;
    ConnectionManager::new(client).await
}

pub async fn check_redis_connection(pool: &mut RedisPool) -> Result<(), RedisError> {
    redis::cmd("PING").query_async(pool).await
}