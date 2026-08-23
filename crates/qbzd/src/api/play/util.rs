use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use qbz_models::QueueTrack;

use crate::api::{err_json, ApiState};
use crate::state::AuthState;

/// Clamp the requested 0-based start index into the resolved list; default 0.
pub(super) fn clamp_index(idx: Option<u64>, len: usize) -> usize {
    match idx {
        Some(i) => (i as usize).min(len.saturating_sub(1)),
        None => 0,
    }
}

/// A compact now-playing summary for the response (title/artist for the CLI's
/// human line; the full state is one `qbzd now` away — §1.1 one-request rule
/// is satisfied because this rides the same response).
pub(super) fn summary(qt: &QueueTrack) -> Value {
    serde_json::json!({
        "id": qt.id,
        "title": qt.title,
        "artist": qt.artist,
        "album": qt.album,
    })
}

/// 409 `needs_auth` — play needs a live Qobuz session (materialization calls
/// the client). Self-contained per-file helper (this crate's convention).
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
