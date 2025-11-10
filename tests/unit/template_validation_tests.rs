use templates_service::models::TemplateType;

#[test]
fn test_template_type_from_str_valid() {
    assert_eq!(
        TemplateType::from_str("email_html"),
        Some(TemplateType::EmailHtml)
    );
    assert_eq!(
        TemplateType::from_str("push_json"),
        Some(TemplateType::PushJson)
    );
}

#[test]
fn test_template_type_from_str_invalid() {
    assert_eq!(TemplateType::from_str("invalid"), None);
    assert_eq!(TemplateType::from_str(""), None);
}

#[test]
fn test_template_type_as_str() {
    assert_eq!(TemplateType::EmailHtml.as_str(), "email_html");
    assert_eq!(TemplateType::PushJson.as_str(), "push_json");
}