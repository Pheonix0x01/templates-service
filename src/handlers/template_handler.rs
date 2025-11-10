use crate::error::AppError;
use crate::models::{ApiResponse, CreateTemplateRequest, TemplateResponse, TemplateType};
use crate::services::{RenderService, TemplateService};
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct GetTemplateQuery {
    pub language: Option<String>,
    pub version: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct RenderQuery {
    pub language: Option<String>,
    pub version: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct RenderRequest {
    pub variables: HashMap<String, Value>,
}

pub async fn create_template(
    service: web::Data<TemplateService>,
    req: web::Json<CreateTemplateRequest>,
) -> Result<HttpResponse, AppError> {
    let template = service.create_template(req.into_inner()).await?;
    
    let response = ApiResponse::success(
        TemplateResponse::from(template),
        "Template created successfully"
    );

    Ok(HttpResponse::Created().json(response))
}

pub async fn get_template(
    service: web::Data<TemplateService>,
    path: web::Path<String>,
    query: web::Query<GetTemplateQuery>,
) -> Result<HttpResponse, AppError> {
    let template_code = path.into_inner();
    
    let template = service.get_template(
        &template_code,
        query.language.as_deref(),
        query.version,
    ).await?;
    
    let response = ApiResponse::success(
        TemplateResponse::from(template),
        "Template retrieved successfully"
    );

    Ok(HttpResponse::Ok().json(response))
}

pub async fn render_template(
    template_service: web::Data<TemplateService>,
    render_service: web::Data<RenderService>,
    path: web::Path<String>,
    query: web::Query<RenderQuery>,
    req: web::Json<RenderRequest>,
) -> Result<HttpResponse, AppError> {
    let template_code = path.into_inner();
    
    let template = template_service.get_template(
        &template_code,
        query.language.as_deref(),
        query.version,
    ).await?;

    let template_type = TemplateType::from_str(&template.template_type)
        .ok_or(AppError::InvalidTemplateType)?;

    let rendered = render_service.render(
        &template.template_code,
        template.version,
        &template.language,
        &template_type,
        &template.content,
        &req.variables,
    ).await?;

    let response = ApiResponse::success(
        rendered,
        "Template rendered successfully"
    );

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_versions(
    service: web::Data<TemplateService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let template_code = path.into_inner();
    
    let templates = service.get_versions(&template_code).await?;
    
    let responses: Vec<TemplateResponse> = templates.into_iter()
        .map(TemplateResponse::from)
        .collect();
    
    let response = ApiResponse::success(
        responses,
        "Template versions retrieved successfully"
    );

    Ok(HttpResponse::Ok().json(response))
}

pub async fn delete_template(
    service: web::Data<TemplateService>,
    path: web::Path<(String, i32)>,
) -> Result<HttpResponse, AppError> {
    let (template_code, version) = path.into_inner();
    
    service.soft_delete(&template_code, version).await?;
    
    let response: ApiResponse<()> = ApiResponse {
        success: true,
        data: None,
        error: None,
        message: "Template deleted successfully".to_string(),
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}