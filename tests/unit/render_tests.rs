use templates_service::rendering::render_template;

#[test]
fn test_render_simple_template() {
    let template = "Hello, {{name}}!";
    let context = serde_json::json!({
        "name": "World"
    });
    
    let result = render_template(template, &context);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, World!");
}

#[test]
fn test_render_with_missing_variable() {
    let template = "Hello, {{name}}!";
    let context = serde_json::json!({});
    
    let result = render_template(template, &context);
    assert!(result.is_ok());
}

#[test]
fn test_render_json_template() {
    let template = r#"{"title": "{{title}}", "body": "{{body}}"}"#;
    let context = serde_json::json!({
        "title": "Test Notification",
        "body": "This is a test"
    });
    
    let result = render_template(template, &context);
    assert!(result.is_ok());
    let rendered = result.unwrap();
    assert!(rendered.contains("Test Notification"));
    assert!(rendered.contains("This is a test"));
}

#[test]
fn test_render_empty_template() {
    let template = "";
    let context = serde_json::json!({});
    
    let result = render_template(template, &context);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[test]
fn test_render_no_variables() {
    let template = "Static content without variables";
    let context = serde_json::json!({});
    
    let result = render_template(template, &context);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Static content without variables");
}