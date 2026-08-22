//! Shared JSON-extraction helpers for Last.fm read endpoints.
//!
//! Last.fm's JSON API is loosely typed (numbers/booleans often arrive as
//! strings), so these small helpers centralize the defensive parsing shared
//! by every `user.*` / `artist.*` / `track.*` read endpoint.

/// Extract the largest image URL (last array entry's `#text`) from a Last.fm object.
/// Returns `None` when the array is missing, empty, or the URL is blank.
pub(super) fn extract_image(value: &serde_json::Value) -> Option<String> {
    value
        .get("image")
        .and_then(|i| i.as_array())
        .and_then(|arr| arr.last())
        .and_then(|last| last.get("#text"))
        .and_then(|t| t.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Extract a non-empty `mbid` field as `Option<String>` (empty strings become `None`).
pub(super) fn extract_mbid(value: &serde_json::Value) -> Option<String> {
    value
        .get("mbid")
        .and_then(|m| m.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Extract a Unix timestamp from a Last.fm `date.uts` field (string or number).
pub(super) fn extract_uts(value: &serde_json::Value) -> Option<i64> {
    value
        .get("date")
        .and_then(|d| d.get("uts"))
        .and_then(|u| {
            u.as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .or_else(|| u.as_i64())
        })
}

/// Parse a `u64` that Last.fm may return as a JSON string or number; defaults to 0.
pub(super) fn parse_u64(value: Option<&serde_json::Value>) -> u64 {
    value
        .and_then(|v| {
            v.as_str()
                .and_then(|s| s.parse::<u64>().ok())
                .or_else(|| v.as_u64())
        })
        .unwrap_or(0)
}
