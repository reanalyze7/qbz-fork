use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::state::AuthState;

use crate::api::{err_json, ApiState};

/// Strict `track_ids` array parse (any non-u64 element is a 400).
pub(super) fn parse_ids(body: &Value) -> Result<Vec<u64>, (String, String)> {
    let hint = "body: {\"id\": 987, \"track_ids\": [176544871]}".to_string();
    let arr = match body.get("track_ids").and_then(|v| v.as_array()) {
        Some(a) if !a.is_empty() => a,
        Some(_) => return Err(("'track_ids' must not be empty".into(), hint)),
        None => return Err(("requires a 'track_ids' array".into(), hint)),
    };
    let mut ids = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        match v.as_u64() {
            Some(id) => ids.push(id),
            None => return Err((format!("track_ids[{i}] is not an unsigned integer"), hint)),
        }
    }
    Ok(ids)
}

pub(super) fn id_param(query: &str) -> Option<u64> {
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if kv.next() == Some("id") {
            return kv.next().and_then(|v| v.parse::<u64>().ok());
        }
    }
    None
}

pub(super) fn auth_gate(state: &ApiState) -> Option<Response<Cursor<Vec<u8>>>> {
    let needs_auth = state
        .shared
        .lock()
        .map(|s| s.auth == AuthState::NeedsAuth)
        .unwrap_or(false);
    if needs_auth {
        Some(err_json(409, "needs_auth", "not logged in to Qobuz", "run: qbzd login"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_param_reads_numeric_id() {
        assert_eq!(id_param("id=987654"), Some(987654));
        assert_eq!(id_param("foo=1&id=42"), Some(42));
        assert_eq!(id_param("id=abc"), None);
        assert_eq!(id_param(""), None);
    }

    #[test]
    fn parse_ids_accepts_valid_and_rejects_bad() {
        assert_eq!(parse_ids(&serde_json::json!({"track_ids": [1, 2, 3]})), Ok(vec![1, 2, 3]));
        assert!(parse_ids(&serde_json::json!({"track_ids": []})).is_err());
        assert!(parse_ids(&serde_json::json!({"track_ids": [1, "x"]})).is_err());
        assert!(parse_ids(&serde_json::json!({})).is_err());
    }
}
