use templates_service::models::{CreateTemplateRequest, TemplateType};
use templates_service::services::{RenderService, TemplateService};
use serde_json::json;
use std::collections::HashMap;

use super::setup;

#[actix_rt::test]
#[serial_test::serial]
async fn test_render_html_template() {
    let ctx = setup::setup_test_env().await;
    let template_service = TemplateService::new(ctx.db_pool.clone(), ctx.redis_pool.clone());
    let render_service = RenderService::new(ctx.redis_pool.clone(), ctx.config.clone());

    let request = CreateTemplateRequest {
        template_code: "render_test".to_string(),
        template_type: "email_html".to_string(),
        language: "en".to_string(),
        content: "<h1>Hello {{name}}</h1>".to_string(),
        meta: None,
    };

    let template = template_service.create_template(request).await.unwrap();

    let mut variables = HashMap::new();
    variables.insert("name".to_string(), json!("Alice"));

    let rendered = render_service
        .render(
            &template.template_code,
            template.version,
            &template.language,
            &TemplateType::EmailHtml,
            &template.content,
            &variables,
        )
        .await
        .unwrap();

    assert_eq!(rendered["rendered"], "<h1>Hello Alice</h1>");

    setup::cleanup_test_env(&ctx.db_pool).await;
}

#[actix_rt::test]
#[serial_test::serial]
async fn test_render_push_json_template() {
    let ctx = setup::setup_test_env().await;
    let template_service = TemplateService::new(ctx.db_pool.clone(), ctx.redis_pool.clone());
    let render_service = RenderService::new(ctx.redis_pool.clone(), ctx.config.clone());

    let request = CreateTemplateRequest {
        template_code: "push_test".to_string(),
        template_type: "push_json".to_string(),
        language: "en".to_string(),
        content: r#"{"title": "Hello {{name}}", "body": "Welcome to our app"}"#.to_string(),
        meta: None,
    };

    let template = template_service.create_template(request).await.unwrap();

    let mut variables = HashMap::new();
    variables.insert("name".to_string(), json!("Bob"));

    let rendered = render_service
        .render(
            &template.template_code,
            template.version,
            &template.language,
            &TemplateType::PushJson,
            &template.content,
            &variables,
        )
        .await
        .unwrap();

    let rendered_obj = rendered["rendered"].as_object().unwrap();
    assert_eq!(rendered_obj["title"], "Hello Bob");
    assert_eq!(rendered_obj["body"], "Welcome to our app");

    setup::cleanup_test_env(&ctx.db_pool).await;
}