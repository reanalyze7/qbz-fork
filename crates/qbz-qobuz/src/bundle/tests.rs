use super::parse::{extract_app_id, extract_bundle_url};

#[test]
fn test_extract_bundle_url() {
    let html = r#"<script src="/resources/7.0.1-b001/bundle.js"></script>"#;
    let result = extract_bundle_url(html);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "/resources/7.0.1-b001/bundle.js");
}

#[test]
fn test_extract_app_id() {
    let bundle = r#"production:{api:{appId:"123456789",appSecret:"abc"}"#;
    let result = extract_app_id(bundle);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "123456789");
}
