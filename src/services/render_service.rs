use crate::cache::RedisPool;
use crate::config::Config;
use crate::error::AppError;
use crate::models::TemplateType;
use redis::AsyncCommands;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;
use tokio::sync::RwLock;

pub struct RenderService {
    redis: RedisPool,
    config: Config,
    compiled_cache: Arc<RwLock<HashMap<String, Tera>>>,
}

impl RenderService {
    pub fn new(redis: RedisPool, config: Config) -> Self {
        Self {
            redis,
            config,
            compiled_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn render(
        &self,
        template_code: &str,
        version: i32,
        language: &str,
        template_type: &TemplateType,
        content: &str,
        variables: &HashMap<String, Value>,
    ) -> Result<Value, AppError> {
        let var_hash = self.hash_variables(variables);
        let cache_key = format!("rendered:{}:{}:{}:{}", template_code, version, language, var_hash);

        let mut redis_conn = self.redis.clone();
        if let Ok(Some(cached)) = redis_conn.get::<_, Option<String>>(&cache_key).await {
            return serde_json::from_str(&cached)
                .map_err(|e| AppError::InternalError(format!("Cache deserialize error: {}", e)));
        }

        let rendered = match template_type {
            TemplateType::EmailHtml => self.render_html(content, variables).await?,
            TemplateType::PushJson => self.render_push_json(content, variables).await?,
        };

        let rendered_str = serde_json::to_string(&rendered)
            .map_err(|e| AppError::InternalError(format!("Serialize error: {}", e)))?;

        let size_kb = rendered_str.len() / 1024;
        if size_kb > self.config.max_rendered_size_kb {
            return Err(AppError::RenderedSizeExceeded);
        }

        let _: Result<(), redis::RedisError> = redis_conn
            .set_ex(&cache_key, &rendered_str, self.config.rendered_cache_ttl_secs)
            .await;

        Ok(rendered)
    }

    async fn render_html(&self, content: &str, variables: &HashMap<String, Value>) -> Result<Value, AppError> {
        let template_key = format!("html_{}", self.hash_content(content));
        
        let mut cache = self.compiled_cache.write().await;
        let tera = cache.entry(template_key.clone()).or_insert_with(|| {
            let mut t = Tera::default();
            t.add_raw_template(&template_key, content).ok();
            t
        });

        let context = tera::Context::from_serialize(variables)
            .map_err(|e| AppError::RenderError(format!("Context error: {}", e)))?;

        let rendered_html = tera
            .render(&template_key, &context)
            .map_err(|e| AppError::RenderError(format!("Tera render error: {}", e)))?;

        Ok(serde_json::json!({ "rendered": rendered_html }))
    }

    async fn render_push_json(&self, content: &str, variables: &HashMap<String, Value>) -> Result<Value, AppError> {
        let template_key = format!("push_{}", self.hash_content(content));
        
        let mut cache = self.compiled_cache.write().await;
        let tera = cache.entry(template_key.clone()).or_insert_with(|| {
            let mut t = Tera::default();
            t.add_raw_template(&template_key, content).ok();
            t
        });

        let context = tera::Context::from_serialize(variables)
            .map_err(|e| AppError::RenderError(format!("Context error: {}", e)))?;

        let rendered_str = tera
            .render(&template_key, &context)
            .map_err(|e| AppError::RenderError(format!("Tera render error: {}", e)))?;

        let rendered_json: Value = serde_json::from_str(&rendered_str)
            .map_err(|e| AppError::RenderError(format!("Invalid JSON after render: {}", e)))?;

        if !rendered_json.is_object() {
            return Err(AppError::RenderError("Rendered push template must be a JSON object".to_string()));
        }

        let obj = rendered_json.as_object().unwrap();
        if !obj.contains_key("title") || !obj.contains_key("body") {
            return Err(AppError::RenderError("Push template must contain 'title' and 'body' fields".to_string()));
        }

        Ok(serde_json::json!({ "rendered": rendered_json }))
    }

    fn hash_variables(&self, variables: &HashMap<String, Value>) -> String {
        let mut hasher = Sha256::new();
        let serialized = serde_json::to_string(variables).unwrap_or_default();
        hasher.update(serialized.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn hash_content(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())[..16].to_string()
    }

    pub async fn invalidate_cache(&self, template_code: &str, version: i32, language: &str) -> Result<(), AppError> {
        let mut redis_conn = self.redis.clone();
        let pattern = format!("rendered:{}:{}:{}:*", template_code, version, language);
        
        let keys: Vec<String> = redis_conn
            .keys(&pattern)
            .await
            .unwrap_or_default();

        for key in keys {
            let _: Result<(), redis::RedisError> = redis_conn.del(&key).await;
        }

        Ok(())
    }
}