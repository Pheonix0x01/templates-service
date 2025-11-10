use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use prometheus::{Encoder, TextEncoder};

mod cache;
mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod services;

use crate::config::Config;
use crate::handlers::{
    create_template, delete_template, get_template, get_versions, health, ready, render_template,
};
use crate::middleware::Metrics;
use crate::services::{RenderService, TemplateService};

async fn metrics_handler() -> HttpResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    let config = Config::from_env();

    tracing::info!("Starting templates-service v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Server will listen on {}", config.server_address());

    let db_pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    tracing::info!("Database connection pool created");

    let redis_pool = cache::create_redis_pool(&config.redis_url)
        .await
        .expect("Failed to create Redis connection");

    tracing::info!("Redis connection established");

    let template_service = web::Data::new(TemplateService::new(
        db_pool.clone(),
        redis_pool.clone(),
    ));

    let render_service = web::Data::new(RenderService::new(redis_pool.clone(), config.clone()));

    let db_data = web::Data::new(db_pool);
    let redis_data = web::Data::new(redis_pool);

    let server_address = config.server_address();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Metrics)
            .app_data(template_service.clone())
            .app_data(render_service.clone())
            .app_data(db_data.clone())
            .app_data(redis_data.clone())
            .route("/health", web::get().to(health))
            .route("/ready", web::get().to(ready))
            .route("/metrics", web::get().to(metrics_handler))
            .service(
                web::scope("/api/v1/templates")
                    .route("/", web::post().to(create_template))
                    .route("/{template_code}", web::get().to(get_template))
                    .route("/{template_code}/render", web::post().to(render_template))
                    .route("/{template_code}/versions", web::get().to(get_versions))
                    .route(
                        "/{template_code}/{version}",
                        web::delete().to(delete_template),
                    ),
            )
    })
    .bind(&server_address)?
    .run()
    .await
}