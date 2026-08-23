use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{err_json, json, ApiState};

use super::errors::runtime_error;

/// `POST /api/playback/seek` (02 §3.3.11). Body `{"position": N}` (absolute)
/// or `{"delta": N}` (additive seconds). Returns the CLAMPED target — the
/// value `Player::seek` will settle on (`qbz-player/src/player/mod.rs:5134`
/// clamps to duration) — rather than a live re-read, since `seek` only sends
/// an async command to the audio thread; the clamp is deterministic so the
/// "post-state" is knowable synchronously.
pub fn seek(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    let player = state.runtime.core().player();
    if player.is_dsd_direct_active() {
        return err_json(
            409,
            "seek_unsupported_dsd",
            "seek is unsupported in DSD-direct mode (bit-perfect passthrough)",
            "set DSD mode to \"convert\": qbzd setup (Audio screen)",
        );
    }
    let ev = player.get_playback_event();
    let target: u64 = if let Some(pos) = body.get("position").and_then(|v| v.as_u64()) {
        pos
    } else if let Some(delta) = body.get("delta").and_then(|v| v.as_i64()) {
        (ev.position as i64 + delta).max(0) as u64
    } else {
        return err_json(
            400,
            "bad_request",
            "seek requires a 'position' or 'delta' field",
            "body: {\"position\": 90} or {\"delta\": -10}",
        );
    };
    let clamped = if ev.duration > 0 { target.min(ev.duration) } else { target };
    if let Err(e) = state.runtime.core().seek(clamped) {
        return runtime_error(&e.to_string());
    }
    json(200, serde_json::json!({"position": clamped, "duration": ev.duration}))
}
