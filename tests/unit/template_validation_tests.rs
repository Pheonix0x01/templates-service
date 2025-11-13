#[test]
fn test_template_name_validation() {
    let valid_names = vec!["welcome_email", "push_notification", "test123"];
    let invalid_names = vec!["", "a", "template with spaces", "template@special"];
    
    for name in valid_names {
        assert!(is_valid_template_name(name), "Expected '{}' to be valid", name);
    }
    
    for name in invalid_names {
        assert!(!is_valid_template_name(name), "Expected '{}' to be invalid", name);
    }
}

#[test]
fn test_template_type_validation() {
    assert!(is_valid_template_type("html"));
    assert!(is_valid_template_type("push_json"));
    assert!(is_valid_template_type("email"));
    assert!(!is_valid_template_type("invalid"));
    assert!(!is_valid_template_type(""));
}

#[test]
fn test_template_content_not_empty() {
    assert!(is_valid_template_content("Some content"));
    assert!(!is_valid_template_content(""));
    assert!(!is_valid_template_content("   "));
}

fn is_valid_template_name(name: &str) -> bool {
    name.len() >= 2 && 
    name.len() <= 100 && 
    name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

fn is_valid_template_type(type_: &str) -> bool {
    matches!(type_, "html" | "push_json" | "email" | "sms")
}

fn is_valid_template_content(content: &str) -> bool {
    !content.trim().is_empty()
}