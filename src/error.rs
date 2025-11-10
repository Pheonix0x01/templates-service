use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    RedisError(redis::RedisError),
    TemplateNotFound,
    RenderError(String),
    InvalidVariables(String),
    InvalidTemplateType,
    InvalidContent(String),
    RenderedSizeExceeded,
    Unauthorized,
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AppError::RedisError(e) => write!(f, "Cache error: {}", e),
            AppError::TemplateNotFound => write!(f, "Template not found"),
            AppError::RenderError(msg) => write!(f, "Render error: {}", msg),
            AppError::InvalidVariables(msg) => write!(f, "Invalid variables: {}", msg),
            AppError::InvalidTemplateType => write!(f, "Invalid template type"),
            AppError::InvalidContent(msg) => write!(f, "Invalid content: {}", msg),
            AppError::RenderedSizeExceeded => write!(f, "Rendered size exceeded limit"),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::TemplateNotFound => StatusCode::NOT_FOUND,
            AppError::RenderError(_) => StatusCode::BAD_REQUEST,
            AppError::InvalidVariables(_) => StatusCode::BAD_REQUEST,
            AppError::InvalidTemplateType => StatusCode::BAD_REQUEST,
            AppError::InvalidContent(_) => StatusCode::BAD_REQUEST,
            AppError::RenderedSizeExceeded => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let error_code = match self {
            AppError::DatabaseError(_) => "db_error",
            AppError::RedisError(_) => "cache_error",
            AppError::TemplateNotFound => "template_not_found",
            AppError::RenderError(_) => "render_error",
            AppError::InvalidVariables(_) => "invalid_variables",
            AppError::InvalidTemplateType => "invalid_template_type",
            AppError::InvalidContent(_) => "invalid_content",
            AppError::RenderedSizeExceeded => "rendered_size_exceeded",
            AppError::Unauthorized => "unauthorized",
            AppError::InternalError(_) => "internal_error",
        };

        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "success": false,
            "data": serde_json::Value::Null,
            "error": error_code,
            "message": self.to_string(),
            "meta": serde_json::Value::Null,
        }))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::RedisError(err)
    }
}