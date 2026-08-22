use super::*;

#[test]
fn test_generate_signature() {
    let sig = generate_signature("test", "params", 1234567890, "secret");
    assert_eq!(sig.len(), 32); // MD5 hex is 32 chars
}

#[test]
fn test_sign_get_file_url() {
    let sig = sign_get_file_url(123456, 27, 1234567890, "testsecret");
    assert_eq!(sig.len(), 32);
}

fn login_response(user_extra: serde_json::Value) -> serde_json::Value {
    let mut user = serde_json::json!({
        "id": 1705826,
        "email": "a@b.c",
        "display_name": "Tester",
        "credential": {"parameters": {"short_label": "Studio"}}
    });
    user.as_object_mut()
        .unwrap()
        .extend(user_extra.as_object().unwrap().clone());
    serde_json::json!({
        "user_auth_token": "token",
        "user": user,
    })
}

#[test]
fn parse_login_response_captures_country_and_language() {
    let response = login_response(serde_json::json!({
        "country_code": "FR",
        "language_code": "fr",
    }));
    let session = parse_login_response(&response).expect("valid login response");
    assert_eq!(session.country_code.as_deref(), Some("FR"));
    assert_eq!(session.language_code.as_deref(), Some("fr"));
}

#[test]
fn parse_login_response_tolerates_missing_country_and_language() {
    // Older captures / partial payloads: both stay None (feature off),
    // the rest of the session parses as before.
    let response = login_response(serde_json::json!({}));
    let session = parse_login_response(&response).expect("valid login response");
    assert_eq!(session.country_code, None);
    assert_eq!(session.language_code, None);
}
