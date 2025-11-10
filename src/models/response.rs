use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<PaginationMeta>,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub limit: i64,
    pub page: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_previous: bool,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: message.into(),
            meta: None,
        }
    }

    pub fn success_with_meta(data: T, message: impl Into<String>, meta: PaginationMeta) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: message.into(),
            meta: Some(meta),
        }
    }
}