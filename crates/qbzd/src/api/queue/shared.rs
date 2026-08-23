use std::io::Cursor;

use qbz_models::QueueTrack;
use tiny_http::Response;

use crate::api::{err_json, ApiState};
use crate::state::AuthState;

/// 409 `needs_auth` — only `add` gates (§3.3.14's Errors column; `list`/
/// `remove`/`clear` carry no needs_auth entry and act on whatever queue
/// already exists). Mirrors `playback::auth_gate` exactly; kept local per
/// this file's self-contained-helpers convention (see api/status.rs's own
/// `bitperfect_label`/`backend_label`).
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

/// Pure pagination slicing (brief step 1) — `GET /api/queue`'s `offset`/
/// `limit` applied to the full `upcoming` list.
pub(super) fn paginate(items: &[QueueTrack], offset: usize, limit: usize) -> Vec<QueueTrack> {
    items.iter().skip(offset).take(limit).cloned().collect()
}

/// `?offset=0&limit=100` (02 §3.3.13). Malformed/missing values fall back to
/// the documented defaults; there is no 400 for a bad query param (a read
/// route degrades gracefully rather than failing the request).
pub(super) fn parse_offset_limit(query: &str) -> (usize, usize) {
    let mut offset = 0usize;
    let mut limit = 100usize;
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let val = kv.next().unwrap_or("");
        match key {
            "offset" => {
                if let Ok(n) = val.parse() {
                    offset = n;
                }
            }
            "limit" => {
                if let Ok(n) = val.parse() {
                    limit = n;
                }
            }
            _ => {}
        }
    }
    (offset, limit)
}
