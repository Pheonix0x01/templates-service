use crate::cache::RedisPool;
use crate::db::DbPool;
use crate::error::AppError;
use crate::models::{CreateTemplateRequest, Template, TemplateType};
use redis::AsyncCommands;
use sqlx::Row;

pub struct TemplateService {
    pool: DbPool,
    redis: RedisPool,
}

impl TemplateService {
    pub fn new(pool: DbPool, redis: RedisPool) -> Self {
        Self { pool, redis }
    }

    pub async fn create_template(&self, req: CreateTemplateRequest) -> Result<Template, AppError> {
        let template_type = TemplateType::from_str(&req.template_type)
            .ok_or(AppError::InvalidTemplateType)?;

        self.validate_content(&template_type, &req.content)?;

        let mut tx = self.pool.begin().await?;

        let max_version: Option<i32> = sqlx::query(
            "SELECT MAX(version) as max_ver FROM templates WHERE template_code = $1 AND language = $2"
        )
        .bind(&req.template_code)
        .bind(&req.language)
        .fetch_optional(&mut *tx)
        .await?
        .and_then(|row| row.try_get("max_ver").ok());

        let new_version = max_version.unwrap_or(0) + 1;

        let template = sqlx::query_as::<_, Template>(
            r#"
            INSERT INTO templates (template_code, version, type, language, content, meta, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, true)
            RETURNING id, template_code, version, type, language, content, created_by, created_at, updated_at, is_active, meta
            "#
        )
        .bind(&req.template_code)
        .bind(new_version)
        .bind(template_type.as_str())
        .bind(&req.language)
        .bind(&req.content)
        .bind(&req.meta)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        self.invalidate_template_cache(&req.template_code, new_version, &req.language).await?;

        Ok(template)
    }

    pub async fn get_template(
        &self,
        template_code: &str,
        language: Option<&str>,
        version: Option<i32>,
    ) -> Result<Template, AppError> {
        let lang = language.unwrap_or("en");

        let cache_key = if let Some(ver) = version {
            format!("template:{}:{}:{}", template_code, ver, lang)
        } else {
            format!("template:{}:latest:{}", template_code, lang)
        };

        let mut redis_conn = self.redis.clone();
        if let Ok(Some(cached)) = redis_conn.get::<_, Option<String>>(&cache_key).await {
            if let Ok(template) = serde_json::from_str::<Template>(&cached) {
                return Ok(template);
            }
        }

        let template = if let Some(ver) = version {
            sqlx::query_as::<_, Template>(
                "SELECT * FROM templates WHERE template_code = $1 AND language = $2 AND version = $3 AND is_active = true"
            )
            .bind(template_code)
            .bind(lang)
            .bind(ver)
            .fetch_optional(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Template>(
                "SELECT * FROM templates WHERE template_code = $1 AND language = $2 AND is_active = true ORDER BY version DESC LIMIT 1"
            )
            .bind(template_code)
            .bind(lang)
            .fetch_optional(&self.pool)
            .await?
        };

        let template = template.ok_or(AppError::TemplateNotFound)?;

        if let Ok(serialized) = serde_json::to_string(&template) {
            let _: Result<(), redis::RedisError> = redis_conn.set_ex(&cache_key, &serialized, 3600).await;
        }

        Ok(template)
    }

    pub async fn get_versions(&self, template_code: &str) -> Result<Vec<Template>, AppError> {
        let templates = sqlx::query_as::<_, Template>(
            "SELECT * FROM templates WHERE template_code = $1 ORDER BY version DESC, language ASC"
        )
        .bind(template_code)
        .fetch_all(&self.pool)
        .await?;

        Ok(templates)
    }

    pub async fn soft_delete(&self, template_code: &str, version: i32) -> Result<(), AppError> {
        let result = sqlx::query(
            "UPDATE templates SET is_active = false WHERE template_code = $1 AND version = $2"
        )
        .bind(template_code)
        .bind(version)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::TemplateNotFound);
        }

        let languages: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT language FROM templates WHERE template_code = $1 AND version = $2"
        )
        .bind(template_code)
        .bind(version)
        .fetch_all(&self.pool)
        .await?;

        for lang in languages {
            self.invalidate_template_cache(template_code, version, &lang).await?;
        }

        Ok(())
    }

    async fn invalidate_template_cache(&self, template_code: &str, version: i32, language: &str) -> Result<(), AppError> {
        let mut redis_conn = self.redis.clone();
        let keys = vec![
            format!("template:{}:{}:{}", template_code, version, language),
            format!("template:{}:latest:{}", template_code, language),
        ];

        for key in keys {
            let _: Result<(), redis::RedisError> = redis_conn.del(&key).await;
        }

        let pattern = format!("rendered:{}:{}:{}:*", template_code, version, language);
        let render_keys: Vec<String> = redis_conn.keys(&pattern).await.unwrap_or_default();
        
        for key in render_keys {
            let _: Result<(), redis::RedisError> = redis_conn.del(&key).await;
        }

        Ok(())
    }

    fn validate_content(&self, template_type: &TemplateType, content: &str) -> Result<(), AppError> {
        match template_type {
            TemplateType::EmailHtml => {
                if content.is_empty() {
                    return Err(AppError::InvalidContent("Content cannot be empty".to_string()));
                }
                Ok(())
            }
            TemplateType::PushJson => {
                let parsed: serde_json::Value = serde_json::from_str(content)
                    .map_err(|e| AppError::InvalidContent(format!("Invalid JSON: {}", e)))?;

                if !parsed.is_object() {
                    return Err(AppError::InvalidContent("Push template must be a JSON object".to_string()));
                }

                Ok(())
            }
        }
    }
}