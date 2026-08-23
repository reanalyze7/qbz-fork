use super::super::*;

#[test]
fn parse_callback_accepts_matching_path_nonce_and_extracts_code() {
    let line = "GET /abc123?code_autorisation=THECODE HTTP/1.1";
    assert_eq!(parse_callback(line, "abc123"), Some("THECODE".to_string()));
}

#[test]
fn parse_callback_falls_back_to_plain_code() {
    let line = "GET /n?code=plaincode HTTP/1.1";
    assert_eq!(parse_callback(line, "n"), Some("plaincode".to_string()));
}

#[test]
fn parse_callback_prefers_code_autorisation_over_code() {
    let line = "GET /n?code=plain&code_autorisation=preferred HTTP/1.1";
    assert_eq!(parse_callback(line, "n"), Some("preferred".to_string()));
}

#[test]
fn parse_callback_rejects_mismatched_path_nonce() {
    // Step 1(b): a wrong path nonce is dropped even with a valid-looking code.
    let line = "GET /WRONG?code_autorisation=THECODE HTTP/1.1";
    assert_eq!(parse_callback(line, "abc123"), None);
}

#[test]
fn parse_callback_rejects_absent_path_nonce() {
    let line = "GET /?code_autorisation=THECODE HTTP/1.1";
    assert_eq!(parse_callback(line, "abc123"), None);
}

#[test]
fn parse_callback_needs_no_state_param_and_ignores_one() {
    // The provider echoing state is exactly what we no longer depend on.
    let no_state = "GET /abc123?code=OK HTTP/1.1";
    assert_eq!(parse_callback(no_state, "abc123"), Some("OK".to_string()));
    let stray_state = "GET /abc123?state=whatever&code=OK HTTP/1.1";
    assert_eq!(parse_callback(stray_state, "abc123"), Some("OK".to_string()));
}

#[test]
fn parse_callback_ignores_browser_noise() {
    assert_eq!(parse_callback("GET /favicon.ico HTTP/1.1", "abc123"), None);
    assert_eq!(parse_callback("", "abc123"), None);
}

#[test]
fn parse_callback_percent_decodes_the_code() {
    let line = "GET /n?code=x%2Fy HTTP/1.1";
    assert_eq!(parse_callback(line, "n"), Some("x/y".to_string()));
}

#[test]
fn code_from_paste_accepts_full_redirect_url_with_path_nonce() {
    let pasted = "http://127.0.0.1:43717/nn?code_autorisation=PASTED";
    assert_eq!(code_from_paste(pasted, "nn"), Some("PASTED".to_string()));
}

#[test]
fn code_from_paste_tolerates_a_missing_path_nonce() {
    // Lenient by design: the operator pasted the URL by hand.
    let pasted = "http://127.0.0.1:43717/?code_autorisation=PASTED";
    assert_eq!(code_from_paste(pasted, "nn"), Some("PASTED".to_string()));
}

#[test]
fn code_from_paste_accepts_a_bare_code() {
    assert_eq!(code_from_paste("JUSTACODE", "nn"), Some("JUSTACODE".to_string()));
    assert_eq!(code_from_paste("  JUSTACODE  ", "nn"), Some("JUSTACODE".to_string()));
}

#[test]
fn code_from_paste_rejects_mismatched_path_nonce_in_url() {
    let pasted = "http://127.0.0.1:43717/WRONG?code_autorisation=PASTED";
    assert_eq!(code_from_paste(pasted, "nn"), None);
}

#[test]
fn code_from_paste_rejects_empty_input() {
    assert_eq!(code_from_paste("   ", "nn"), None);
}

#[test]
fn gen_nonce_is_long_unique_and_hex() {
    let a = gen_nonce();
    let b = gen_nonce();
    assert_ne!(a, b, "two nonces collided");
    assert_eq!(a.len(), 48, "nonce length: {}", a.len());
    assert!(a.chars().all(|c| c.is_ascii_hexdigit()), "non-hex: {a}");
}
