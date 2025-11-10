use tera::Tera;

#[test]
fn test_render_simple_html() {
    let mut tera = Tera::default();
    tera.add_raw_template("test.html", "<h1>Hello {{name}}</h1>")
        .unwrap();

    let mut context = tera::Context::new();
    context.insert("name", "World");

    let result = tera.render("test.html", &context).unwrap();
    assert_eq!(result, "<h1>Hello World</h1>");
}

#[test]
fn test_render_html_escapes_dangerous_input() {
    let mut tera = Tera::default();
    tera.add_raw_template("test.html", "<p>{{content}}</p>").unwrap();

    let mut context = tera::Context::new();
    context.insert("content", "<script>alert('xss')</script>");

    let result = tera.render("test.html", &context).unwrap();
    assert!(result.contains("&lt;script&gt;"));
    assert!(!result.contains("<script>"));
}

#[test]
fn test_render_push_json() {
    let mut tera = Tera::default();
    let template = r#"{"title": "Hello {{name}}", "body": "Welcome {{name}}"}"#;
    tera.add_raw_template("test.json", template).unwrap();

    let mut context = tera::Context::new();
    context.insert("name", "Alice");

    let result = tera.render("test.json", &context).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed["title"], "Hello Alice");
    assert_eq!(parsed["body"], "Welcome Alice");
}

#[test]
fn test_render_push_json_validates_structure() {
    let template = r#"{"title": "{{title}}", "body": "{{body}}"}"#;
    let mut tera = Tera::default();
    tera.add_raw_template("test.json", template).unwrap();

    let mut context = tera::Context::new();
    context.insert("title", "Test Title");
    context.insert("body", "Test Body");

    let result = tera.render("test.json", &context).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert!(parsed.is_object());
    assert!(parsed.get("title").is_some());
    assert!(parsed.get("body").is_some());
}