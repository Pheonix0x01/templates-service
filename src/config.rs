use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub max_rendered_size_kb: usize,
    pub template_cache_ttl_secs: u64,
    pub rendered_cache_ttl_secs: u64,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL")
                .expect("REDIS_URL must be set"),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid u16"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            max_rendered_size_kb: env::var("MAX_RENDERED_SIZE_KB")
                .unwrap_or_else(|_| "64".to_string())
                .parse()
                .expect("MAX_RENDERED_SIZE_KB must be a valid number"),
            template_cache_ttl_secs: env::var("TEMPLATE_CACHE_TTL_SECS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .expect("TEMPLATE_CACHE_TTL_SECS must be a valid number"),
            rendered_cache_ttl_secs: env::var("RENDERED_CACHE_TTL_SECS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .expect("RENDERED_CACHE_TTL_SECS must be a valid number"),
        }
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}