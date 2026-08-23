use std::io::Cursor;
use std::sync::Arc;

use tiny_http::Response;

use crate::api::{json, ApiState};

use super::cold_start::cold_start;
use super::errors::{auth_gate, runtime_error};
use super::resolve_quality;

/// `POST /api/playback/play` (02 §3.3.5). Resume if paused; cold-start the
/// current queue track when `!has_loaded_audio()` (the desktop's
/// `toggle_play_pause` cold-start branch, `crates/qbz/src/playback.rs:3837-3860`).
pub fn play(state: &ApiState) -> Response<Cursor<Vec<u8>>> {
    let player = state.runtime.core().player();
    if player.has_loaded_audio() {
        return match state.runtime.core().resume() {
            Ok(()) => json(200, serde_json::json!({"state": "playing"})),
            Err(e) => runtime_error(&e.to_string()),
        };
    }
    // The cold-start load is spawn-and-ack: report "loading" (honest ack);
    // `qbzd now` / SSE show the transition to playing.
    match cold_start(state) {
        Ok(()) => json(200, serde_json::json!({"state": "loading"})),
        Err(resp) => resp,
    }
}

/// `POST /api/playback/pause` (02 §3.3.6). Never cold-starts; exit set is
/// 0 · 1 · 3 (no 5, §2.2) so a `Player::pause` channel failure is
/// [`runtime_error`], not a device error.
pub fn pause(state: &ApiState) -> Response<Cursor<Vec<u8>>> {
    match state.runtime.core().pause() {
        Ok(()) => json(200, serde_json::json!({"state": "paused"})),
        Err(e) => runtime_error(&e.to_string()),
    }
}

/// `POST /api/playback/toggle` (02 §3.3.7). Mirrors the desktop's
/// `toggle_play_pause`: playing -> pause; paused-with-loaded-audio -> resume;
/// nothing loaded -> cold-start (same gate as `play`). Exit 5 is reserved for
/// the cold-start branch (`cold_start`'s own device error) — the
/// pause/resume branches use [`runtime_error`] like plain `pause`/`stop`.
pub fn toggle(state: &ApiState) -> Response<Cursor<Vec<u8>>> {
    let player = state.runtime.core().player();
    let ev = player.get_playback_event();
    if ev.is_playing {
        return match state.runtime.core().pause() {
            Ok(()) => json(200, serde_json::json!({"state": "paused"})),
            Err(e) => runtime_error(&e.to_string()),
        };
    }
    if player.has_loaded_audio() {
        return match state.runtime.core().resume() {
            Ok(()) => json(200, serde_json::json!({"state": "playing"})),
            Err(e) => runtime_error(&e.to_string()),
        };
    }
    // The cold-start load is spawn-and-ack: report "loading" (honest ack);
    // `qbzd now` / SSE show the transition to playing.
    match cold_start(state) {
        Ok(()) => json(200, serde_json::json!({"state": "loading"})),
        Err(resp) => resp,
    }
}

/// `POST /api/playback/stop` (02 §3.3.8). Never cold-starts; same exit-set
/// reasoning as `pause`.
pub fn stop(state: &ApiState) -> Response<Cursor<Vec<u8>>> {
    match state.runtime.core().stop() {
        Ok(()) => json(200, serde_json::json!({"state": "stopped"})),
        Err(e) => runtime_error(&e.to_string()),
    }
}

/// `POST /api/playback/next` (02 §3.3.9).
pub fn next(state: &ApiState) -> Response<Cursor<Vec<u8>>> {
    advance(state, true)
}

/// `POST /api/playback/previous` (02 §3.3.10).
pub fn previous(state: &ApiState) -> Response<Cursor<Vec<u8>>> {
    advance(state, false)
}

/// `next`/`previous` (02 §3.3.9-10): gate on NeedsAuth BEFORE running the
/// ritual (unconditional per those two rows' Errors column, unlike
/// play/toggle's cold-start-only gate), then SPAWN
/// `qbz_app::playback_driver::advance_and_play` — the FULL ritual (skip-walk →
/// play → prefetch → persist), never a bare cursor move (02 §2.2 trap).
///
/// Spawn-and-ack: the API serve loop is single-threaded, and the ritual's
/// load leg (resolve+fetch) can block for many seconds on a slow link — an
/// inline `block_on` starved every other route ("daemon not reachable" while
/// the action DID execute). The ritual runs on the tokio runtime; a failure
/// latches into `last_errors.stream` (visible via `qbzd status`) and the log.
/// The landing track is no longer reported synchronously — follow it via
/// `qbzd now` / SSE.
fn advance(state: &ApiState, forward: bool) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let quality = resolve_quality(state);
    let runtime = std::sync::Arc::clone(&state.runtime);
    let shared = Arc::clone(&state.shared);
    state.rt.spawn(async move {
        if let Err(err) =
            qbz_app::playback_driver::advance_and_play(runtime.as_ref(), quality, forward).await
        {
            log::error!("[api] advance(forward={forward}) failed: {err}");
            if let Ok(mut s) = shared.lock() {
                s.last_errors.stream = Some(format!("advance: {err}"));
            }
        }
    });
    json(
        200,
        serde_json::json!({"queued": true, "direction": if forward { "next" } else { "previous" }}),
    )
}
