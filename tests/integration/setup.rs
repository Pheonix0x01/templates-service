use sqlx::PgPool;
use templates_service::cache::{create_redis_pool, RedisPool};
use templates_service::config::Config;

pub struct TestContext {
    pub db_pool: PgPool,
    pub redis_pool: RedisPool,
    pub config: Config,
}

pub async fn setup_test_env() -> TestContext {
    let config = Config::from_env();

    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    let redis_pool = create_redis_pool(&config.redis_url)
        .await
        .expect("Failed to connect to Redis");

    TestContext {
        db_pool,
        redis_pool,
        config,
    }
}

pub async fn cleanup_test_env(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE templates CASCADE")
        .execute(pool)
        .await
        .expect("Failed to cleanup test data");
}