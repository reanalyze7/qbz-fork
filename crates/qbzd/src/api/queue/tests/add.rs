use serde_json::Value;

use crate::api::queue::add::{parse_position, parse_track_ids, AddPosition};

#[test]
fn parse_track_ids_accepts_a_valid_array() {
    let body = serde_json::json!({"track_ids": [176544872, 176544873]});
    assert_eq!(parse_track_ids(&body), Ok(vec![176544872, 176544873]));
}

#[test]
fn parse_track_ids_rejects_a_mixed_valid_body_naming_the_position() {
    // Review fix: `filter_map` silently dropped the malformed element —
    // "add 3, enqueue 2". Now the WHOLE body is refused (400) before any
    // core call, so the queue stays untouched (`add` parses first,
    // resolves/mutates only on Ok).
    let body = serde_json::json!({"track_ids": [176544872, "oops", 176544873]});
    let (message, _hint) = parse_track_ids(&body).unwrap_err();
    assert_eq!(message, "track_ids[1] is not an unsigned integer track id");
}

#[test]
fn parse_track_ids_rejects_negative_and_fractional_ids() {
    let neg = serde_json::json!({"track_ids": [-1]});
    assert!(parse_track_ids(&neg).is_err());
    let frac = serde_json::json!({"track_ids": [1.5]});
    assert!(parse_track_ids(&frac).is_err());
}

#[test]
fn parse_track_ids_rejects_an_empty_array() {
    let body = serde_json::json!({"track_ids": []});
    let (message, _hint) = parse_track_ids(&body).unwrap_err();
    assert_eq!(message, "'track_ids' must not be empty");
}

#[test]
fn parse_track_ids_rejects_a_missing_or_non_array_field() {
    assert!(parse_track_ids(&serde_json::json!({})).is_err());
    assert!(parse_track_ids(&serde_json::json!({"track_ids": 176544872})).is_err());
    assert!(parse_track_ids(&Value::Null).is_err());
}

#[test]
fn parse_position_defaults_end_and_matches_the_two_literals() {
    assert_eq!(parse_position(&serde_json::json!({})), Ok(AddPosition::End));
    assert_eq!(
        parse_position(&serde_json::json!({"position": "end"})),
        Ok(AddPosition::End)
    );
    assert_eq!(
        parse_position(&serde_json::json!({"position": "next"})),
        Ok(AddPosition::Next)
    );
}

#[test]
fn parse_position_rejects_unknown_literals_and_non_strings() {
    // Strict match (review fix): a typo must not silently become "end".
    let (message, _hint) =
        parse_position(&serde_json::json!({"position": "nxet"})).unwrap_err();
    assert_eq!(message, "invalid position \"nxet\" — use \"end\" or \"next\"");
    assert!(parse_position(&serde_json::json!({"position": 3})).is_err());
    assert!(parse_position(&serde_json::json!({"position": null})).is_err());
}
