use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{err_json, json, ApiState};

/// `POST /api/queue/clear` (02 §3.3.16). Body `{"keep_current": bool}`
/// (default `true` when the field is absent — the CLI always sends it
/// explicitly, §"queue clear" in cli/queue.rs).
pub fn clear(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    let keep_current = body.get("keep_current").and_then(|v| v.as_bool()).unwrap_or(true);
    state.rt.block_on(state.runtime.core().clear_queue(keep_current));
    let total_tracks = state.rt.block_on(state.runtime.core().get_queue_state()).total_tracks;
    json(200, serde_json::json!({"total_tracks": total_tracks}))
}

/// `POST /api/queue/move` (CONSOLE). Body `{"from": N, "to": N}` (0-based).
/// GUI drag-reorder (core `move_track`). 404 when either index is out of range.
pub fn reorder(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    let from = match body.get("from").and_then(|v| v.as_u64()) {
        Some(n) => n as usize,
        None => return err_json(400, "bad_request", "move requires 'from' and 'to'", "body: {\"from\": 7, \"to\": 2}"),
    };
    let to = match body.get("to").and_then(|v| v.as_u64()) {
        Some(n) => n as usize,
        None => return err_json(400, "bad_request", "move requires 'from' and 'to'", "body: {\"from\": 7, \"to\": 2}"),
    };
    if state.rt.block_on(state.runtime.core().move_track(from, to)) {
        json(200, serde_json::json!({"from": from, "to": to}))
    } else {
        err_json(404, "not_found", "queue index out of range", "check: qbzd queue list")
    }
}
