use super::redact::redact_id_like;
use super::rows::match_status;

#[test]
fn redacts_uuid_and_long_hex() {
    let s = "session 550e8400-e29b-41d4-a716-446655440000 token \
             0123456789abcdef0123456789abcdef ok";
    let out = redact_id_like(s);
    assert!(out.contains("<uuid>"), "{out}");
    assert!(out.contains("<hex>"), "{out}");
    assert!(out.contains("session"));
    assert!(out.contains("ok"));
}

#[test]
fn leaves_short_hex_alone() {
    // 8 hex chars (a SONAME-ish short id) is below the 32-char threshold.
    assert_eq!(redact_id_like("abc123 deadbeef end"), "abc123 deadbeef end");
}

#[test]
fn match_status_rules() {
    assert_eq!(match_status("ON", "ON"), 1);
    assert_eq!(match_status("ON", "OFF"), 2);
    assert_eq!(match_status("—", "ON"), 0);
    assert_eq!(match_status("ON", "—"), 0);
}
