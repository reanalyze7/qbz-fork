use std::collections::HashMap;

use super::{DEFAULT_LIMIT, MAX_LIMIT};

/// Percent-decoded query-string map (values only; keys are plain ascii).
pub(super) fn parse(query: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut kv = pair.splitn(2, '=');
        let key = kv.next().unwrap_or("").to_string();
        let raw = kv.next().unwrap_or("");
        let val = urlencoding::decode(raw)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| raw.to_string());
        m.insert(key, val);
    }
    m
}

/// `limit` (clamped 1..=MAX, default 20) + `offset` (default 0).
pub(super) fn limit_offset(p: &HashMap<String, String>) -> (u32, u32) {
    let limit = p
        .get("limit")
        .and_then(|v| v.parse::<u32>().ok())
        .map(|n| n.clamp(1, MAX_LIMIT))
        .unwrap_or(DEFAULT_LIMIT);
    let offset = p.get("offset").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
    (limit, offset)
}

/// A boolean-ish query flag (`?suggest=1` / `?suggest=true`).
pub(super) fn wants(p: &HashMap<String, String>, key: &str) -> bool {
    matches!(p.get(key).map(String::as_str), Some("1") | Some("true"))
}
