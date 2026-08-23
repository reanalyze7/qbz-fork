use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{err_json, json, ApiState};

use super::shared::auth_gate;

/// `POST /api/queue/jump` (CONSOLE). Body `{"index": N}` (0-based). A
/// click-to-play-row: moves the cursor (`play_index`) AND starts audio through
/// the shipped ritual — never a bare cursor move (control-surface §2.2).
/// Auth-gated (needs a session to resolve the stream). The cursor move is
/// synchronous (the 404 contract is immediate); the resolve+play leg is
/// spawn-and-ack (see `playback::advance`) so a slow fetch can't starve the
/// single-threaded API — failures latch into `last_errors.stream`.
pub fn jump(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let index = match body.get("index").and_then(|v| v.as_u64()) {
        Some(n) => n as usize,
        None => return err_json(400, "bad_request", "jump requires an 'index'", "body: {\"index\": 2}"),
    };
    let track = match state.rt.block_on(state.runtime.core().play_index(index)) {
        Some(t) => t,
        None => return err_json(404, "not_found", &format!("queue index {index} is out of range"), "check: qbzd queue list"),
    };
    let track_id = track.id;
    let quality = super::super::playback::resolve_quality(state);
    let runtime = std::sync::Arc::clone(&state.runtime);
    let shared = std::sync::Arc::clone(&state.shared);
    state.rt.spawn(async move {
        if let Err(err) = runtime
            .core()
            .play_track_resolved(track_id, quality, None, None, 0)
            .await
        {
            log::error!("[api] queue jump play of {track_id} failed: {err}");
            if let Ok(mut s) = shared.lock() {
                s.last_errors.stream = Some(format!("jump: {err}"));
            }
            return;
        }
        qbz_app::playback_driver::save_session_now(runtime.as_ref()).await;
    });
    json(
        200,
        serde_json::json!({"playing": index, "track": {"id": track.id, "title": track.title, "artist": track.artist}}),
    )
}

/// `POST /api/queue/stop-after` (CONSOLE). Body `{"track_id": N}` |
/// `{"current": true}` | `{"off": true}`. Sets/clears the stop-after gate
/// (core `set_stop_after`/`clear_stop_after`).
pub fn stop_after(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    if body.get("off").and_then(|v| v.as_bool()).unwrap_or(false) {
        state.rt.block_on(state.runtime.core().clear_stop_after());
        return json(200, serde_json::json!({"stop_after_track_id": Value::Null}));
    }
    let track_id = if body.get("current").and_then(|v| v.as_bool()).unwrap_or(false) {
        match state.rt.block_on(state.runtime.core().get_queue_state()).current_track {
            Some(t) => t.id,
            None => return err_json(404, "not_found", "nothing is playing", "queue a track first"),
        }
    } else if let Some(id) = body.get("track_id").and_then(|v| v.as_u64()) {
        id
    } else {
        return err_json(
            400,
            "bad_request",
            "stop-after requires track_id, current, or off",
            "body: {\"current\": true} | {\"track_id\": N} | {\"off\": true}",
        );
    };
    state.rt.block_on(state.runtime.core().set_stop_after(track_id));
    json(200, serde_json::json!({"stop_after_track_id": track_id}))
}
