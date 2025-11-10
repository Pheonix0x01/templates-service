use templates_service::models::CreateTemplateRequest;
use templates_service::services::TemplateService;

use super::setup;

#[actix_rt::test]
#[serial_test::serial]
async fn test_create_and_fetch_template() {
    let ctx = setup::setup_test_env().await;
    let service = TemplateService::new(ctx.db_pool.clone(), ctx.redis_pool.clone());

    let request = CreateTemplateRequest {
        template_code: "test_welcome".to_string(),
        template_type: "email_html".to_string(),
        language: "en".to_string(),
        content: "<h1>Welcome {{name}}</h1>".to_string(),
        meta: None,
    };

    let created = service.create_template(request).await.unwrap();

    assert_eq!(created.template_code, "test_welcome");
    assert_eq!(created.version, 1);
    assert_eq!(created.language, "en");

    let fetched = service
        .get_template("test_welcome", Some("en"), None)
        .await
        .unwrap();

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.content, "<h1>Welcome {{name}}</h1>");

    setup::cleanup_test_env(&ctx.db_pool).await;
}

#[actix_rt::test]
#[serial_test::serial]
async fn test_template_versioning() {
    let ctx = setup::setup_test_env().await;
    let service = TemplateService::new(ctx.db_pool.clone(), ctx.redis_pool.clone());

    let request1 = CreateTemplateRequest {
        template_code: "versioned_template".to_string(),
        template_type: "email_html".to_string(),
        language: "en".to_string(),
        content: "<h1>Version 1</h1>".to_string(),
        meta: None,
    };

    let v1 = service.create_template(request1).await.unwrap();
    assert_eq!(v1.version, 1);

    let request2 = CreateTemplateRequest {
        template_code: "versioned_template".to_string(),
        template_type: "email_html".to_string(),
        language: "en".to_string(),
        content: "<h1>Version 2</h1>".to_string(),
        meta: None,
    };

    let v2 = service.create_template(request2).await.unwrap();
    assert_eq!(v2.version, 2);

    let versions = service.get_versions("versioned_template").await.unwrap();
    assert_eq!(versions.len(), 2);

    setup::cleanup_test_env(&ctx.db_pool).await;
}

#[actix_rt::test]
#[serial_test::serial]
async fn test_soft_delete_template() {
    let ctx = setup::setup_test_env().await;
    let service = TemplateService::new(ctx.db_pool.clone(), ctx.redis_pool.clone());

    let request = CreateTemplateRequest {
        template_code: "delete_test".to_string(),
        template_type: "email_html".to_string(),
        language: "en".to_string(),
        content: "<h1>To be deleted</h1>".to_string(),
        meta: None,
    };

    let created = service.create_template(request).await.unwrap();

    service
        .soft_delete("delete_test", created.version)
        .await
        .unwrap();

    let result = service.get_template("delete_test", Some("en"), None).await;
    assert!(result.is_err());

    setup::cleanup_test_env(&ctx.db_pool).await;
}