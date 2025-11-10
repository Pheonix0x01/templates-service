use crate::cache::RedisPool;
use crate::db::DbPool;
use actix_web::{web, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
struct ReadyResponse {
    status: String,
    database: String,
    redis: String,
}

pub async fn health() -> HttpResponse {
    let response = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    HttpResponse::Ok().json(response)
}

pub async fn ready(
    db_pool: web::Data<DbPool>,
    redis_pool: web::Data<RedisPool>,
) -> HttpResponse {
    let db_status = match crate::db::check_connection(&db_pool).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let mut redis_conn = redis_pool.get_ref().clone();
    let redis_status = match crate::cache::check_redis_connection(&mut redis_conn).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let overall_status = if db_status == "connected" && redis_status == "connected" {
        "ready"
    } else {
        "not_ready"
    };

    let response = ReadyResponse {
        status: overall_status.to_string(),
        database: db_status.to_string(),
        redis: redis_status.to_string(),
    };

    if overall_status == "ready" {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}