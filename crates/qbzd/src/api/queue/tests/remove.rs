use crate::api::queue::remove::{check_remove_index, parse_remove_index, RemoveCheck};
use crate::api::response::error_body;

#[test]
fn parse_remove_index_distinguishes_missing_from_wrong_type() {
    // Review fix: a present-but-wrong-type `index` gets its own message,
    // not the missing-field copy.
    let (missing, _) = parse_remove_index(&serde_json::json!({})).unwrap_err();
    assert_eq!(missing, "remove requires an 'index' field");
    let (wrong, _) = parse_remove_index(&serde_json::json!({"index": "3"})).unwrap_err();
    assert_eq!(wrong, "'index' must be a non-negative integer");
    let (neg, _) = parse_remove_index(&serde_json::json!({"index": -1})).unwrap_err();
    assert_eq!(neg, "'index' must be a non-negative integer");
    assert_eq!(parse_remove_index(&serde_json::json!({"index": 3})), Ok(3));
}

// -------------------------- remove-index gate --------------------------

#[test]
fn check_remove_index_ok_for_a_non_playing_in_range_index() {
    assert_eq!(check_remove_index(2, 14, Some(1)), RemoveCheck::Ok);
}

#[test]
fn check_remove_index_rejects_out_of_range() {
    assert_eq!(check_remove_index(14, 14, Some(1)), RemoveCheck::OutOfRange);
    assert_eq!(check_remove_index(100, 14, None), RemoveCheck::OutOfRange);
}

#[test]
fn check_remove_index_rejects_the_playing_index() {
    // 02 §3.3.15's own example state: current_index=1, removing index 1.
    assert_eq!(check_remove_index(1, 14, Some(1)), RemoveCheck::PlayingIndex);
}

#[test]
fn remove_playing_index_error_body_matches_the_documented_hint() {
    // 02 §3.3.15 verbatim: {"error":{"code":"bad_request",
    // "message":"index 1 is the playing track","hint":"use: qbzd next, or qbzd queue clear"}}
    let body = error_body(
        "bad_request",
        "index 1 is the playing track",
        "use: qbzd next, or qbzd queue clear",
    );
    assert_eq!(body["error"]["code"], "bad_request");
    assert_eq!(body["error"]["message"], "index 1 is the playing track");
    assert_eq!(body["error"]["hint"], "use: qbzd next, or qbzd queue clear");
}
