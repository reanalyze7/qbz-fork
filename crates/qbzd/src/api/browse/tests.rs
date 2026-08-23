use std::collections::HashMap;

use super::query::{limit_offset, parse, wants};
use super::{DEFAULT_LIMIT, MAX_LIMIT};

#[test]
fn parse_decodes_values_and_splits_pairs() {
    let m = parse("id=abc&view=top&q=a%20b");
    assert_eq!(m.get("id").unwrap(), "abc");
    assert_eq!(m.get("view").unwrap(), "top");
    assert_eq!(m.get("q").unwrap(), "a b");
}

#[test]
fn limit_offset_clamps_and_defaults() {
    let mut m = HashMap::new();
    assert_eq!(limit_offset(&m), (DEFAULT_LIMIT, 0));
    m.insert("limit".into(), "500".into());
    m.insert("offset".into(), "5".into());
    assert_eq!(limit_offset(&m), (MAX_LIMIT, 5));
    m.insert("limit".into(), "0".into());
    assert_eq!(limit_offset(&m).0, 1);
}

#[test]
fn wants_reads_truthy_flags() {
    let mut m = HashMap::new();
    assert!(!wants(&m, "suggest"));
    m.insert("suggest".into(), "1".into());
    assert!(wants(&m, "suggest"));
    m.insert("suggest".into(), "true".into());
    assert!(wants(&m, "suggest"));
    m.insert("suggest".into(), "0".into());
    assert!(!wants(&m, "suggest"));
}
