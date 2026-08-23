use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{json, ApiState};

use super::mapping::repeat_str;
use super::shared::{paginate, parse_offset_limit};

/// `GET /api/queue` (02 §3.3.13). `query` is the raw query string (no leading
/// `?`) — `route()` strips it off the path before dispatch, so it is threaded
/// through separately. Reads the FULL queue state (`get_queue_state_full`,
/// not the UI-capped `get_queue_state`) so `offset`/`limit` paginate over the
/// complete `upcoming` list rather than an already-truncated 20-entry window.
///
/// ADDITIVE `history` field (§3.1.4 allows additive within api_version 1):
/// the full played-track list, recent-first (`QueueManager::get_state_full`'s
/// convention, qbz-player/src/queue.rs:1064-1070). Without it the §2.2
/// `queue list` example — a played row rendered ABOVE the current track — is
/// unreproducible from the response. `history_len` stays exactly as
/// documented (§3.3.13); nothing existing is renamed or removed.
pub fn list(state: &ApiState, query: &str) -> Response<Cursor<Vec<u8>>> {
    let (offset, limit) = parse_offset_limit(query);
    let queue = state.rt.block_on(state.runtime.core().get_queue_state_full());

    let current_track = queue
        .current_track
        .as_ref()
        .map(|t| serde_json::to_value(t).unwrap_or(Value::Null))
        .unwrap_or(Value::Null);
    let upcoming: Vec<Value> = paginate(&queue.upcoming, offset, limit)
        .iter()
        .map(|t| serde_json::to_value(t).unwrap_or(Value::Null))
        .collect();
    let history: Vec<Value> = queue
        .history
        .iter()
        .map(|t| serde_json::to_value(t).unwrap_or(Value::Null))
        .collect();

    json(
        200,
        serde_json::json!({
            "current_track": current_track,
            "current_index": queue.current_index,
            "upcoming": upcoming,
            "history": history,
            "history_len": queue.history.len(),
            "shuffle": queue.shuffle,
            "repeat": repeat_str(queue.repeat),
            "total_tracks": queue.total_tracks,
            "stop_after_track_id": queue.stop_after_track_id,
            "offset": offset,
            "limit": limit,
        }),
    )
}
