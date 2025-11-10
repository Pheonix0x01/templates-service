use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateType {
    EmailHtml,
    PushJson,
}

impl TemplateType {
    pub fn as_str(&self) -> &str {
        match self {
            TemplateType::EmailHtml => "email_html",
            TemplateType::PushJson => "push_json",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "email_html" => Some(TemplateType::EmailHtml),
            "push_json" => Some(TemplateType::PushJson),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Template {
    pub id: Uuid,
    pub template_code: String,
    pub version: i32,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub template_type: String,
    pub language: String,
    pub content: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub template_code: String,
    #[serde(rename = "type")]
    pub template_type: String,
    pub language: String,
    pub content: String,
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub template_code: String,
    pub version: i32,
    #[serde(rename = "type")]
    pub template_type: String,
    pub language: String,
    pub content: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub meta: Option<serde_json::Value>,
}

impl From<Template> for TemplateResponse {
    fn from(t: Template) -> Self {
        Self {
            id: t.id,
            template_code: t.template_code,
            version: t.version,
            template_type: t.template_type,
            language: t.language,
            content: t.content,
            created_by: t.created_by,
            created_at: t.created_at,
            updated_at: t.updated_at,
            is_active: t.is_active,
            meta: t.meta,
        }
    }
}