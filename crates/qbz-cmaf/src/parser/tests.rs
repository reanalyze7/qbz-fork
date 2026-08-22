use super::init_payload::parse_init_uuid_payload;

#[test]
fn truncated_raw_len_is_error() {
    // Build a minimal payload that claims more raw bytes than available:
    // raw_len sits at bytes 26..28, raw data starts at 28, so a 30-byte
    // payload with raw_len=100 must fail.
    let mut p = vec![0u8; 30];
    // set raw_len = 100 at bytes 26..28
    p[26] = 0;
    p[27] = 100;
    match parse_init_uuid_payload(&p) {
        Ok(_) => panic!("expected truncation error"),
        Err(e) => {
            let msg = format!("{e}");
            assert!(msg.contains("truncated") || msg.contains("raw"), "{msg}");
        }
    }
}
