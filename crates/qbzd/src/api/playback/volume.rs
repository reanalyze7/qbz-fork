use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{canon_volume, err_json, json, ApiState};

use super::errors::runtime_error;

/// `POST /api/playback/volume` (02 §3.3.12). One of three body forms:
/// `{"volume": F}` (absolute 0.0-1.0), `{"delta": F}` (additive), or
/// `{"mute": "on"|"off"|"toggle"}` (also `qbzd mute`'s route — no dedicated
/// route, §2.2). All three are gated by the same DSD-direct guard as `seek`.
pub fn volume(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    let player = state.runtime.core().player();
    if player.is_dsd_direct_active() {
        return err_json(
            409,
            "volume_fixed_dsd",
            "volume is fixed in DSD-direct mode (bit-perfect passthrough)",
            "set DSD mode to \"convert\": qbzd setup (Audio screen)",
        );
    }
    let live = player.get_playback_event().volume;

    if let Some(mute_arg) = body.get("mute").and_then(|v| v.as_str()) {
        return apply_mute(state, live, mute_arg);
    }

    let (muted_before, nominal_before) = nominal_volume(state, live);
    let target = if let Some(v) = body.get("volume").and_then(|v| v.as_f64()) {
        (v as f32).clamp(0.0, 1.0)
    } else if let Some(d) = body.get("delta").and_then(|v| v.as_f64()) {
        (nominal_before + d as f32).clamp(0.0, 1.0)
    } else {
        return err_json(
            400,
            "bad_request",
            "volume requires a 'volume', 'delta' or 'mute' field",
            "body: {\"volume\": 0.75}",
        );
    };

    // An explicit non-zero target clears an active mute (desktop parity:
    // `crates/qbz/src/playback.rs:3921-3924` — "a non-zero level clears any
    // active mute").
    let mut muted_after = muted_before;
    if target > 0.0 && muted_before {
        if let Ok(mut s) = state.shared.lock() {
            s.muted = false;
        }
        muted_after = false;
    }
    if let Err(e) = state.runtime.core().set_volume(target) {
        return runtime_error(&e.to_string());
    }
    json(200, serde_json::json!({"volume": canon_volume(target), "muted": muted_after}))
}

/// `{"mute": "on"|"off"|"toggle"}` — stash-then-zero / restore, mirroring the
/// desktop's `toggle_mute` (`crates/qbz/src/playback.rs:3936-3961`) but
/// against `DaemonShared` instead of process statics. `live` is the player's
/// volume BEFORE this call (the value to stash on a fresh mute).
fn apply_mute(state: &ApiState, live: f32, arg: &str) -> Response<Cursor<Vec<u8>>> {
    let mute_on = match arg {
        "on" => true,
        "off" => false,
        "toggle" => !state.shared.lock().map(|s| s.muted).unwrap_or(false),
        other => {
            return err_json(
                400,
                "bad_request",
                &format!("invalid mute state '{other}' — use on, off, or toggle"),
                "body: {\"mute\": \"toggle\"}",
            )
        }
    };

    let mut guard = match state.shared.lock() {
        Ok(g) => g,
        Err(_) => {
            return err_json(500, "internal", "daemon state lock poisoned", "restart qbzd")
        }
    };

    // Desktop fallback for a never-set / zero premute level (playback.rs:3944,
    // 3956): 0.7, so a mute taken at volume 0 still restores to something
    // audible on unmute.
    let (nominal, set_result) = if mute_on && !guard.muted {
        let stash = if live > 0.0 { live } else { 0.7 };
        guard.premute_volume = stash;
        guard.muted = true;
        (stash, state.runtime.core().set_volume(0.0))
    } else if !mute_on && guard.muted {
        let restored = if guard.premute_volume > 0.0 { guard.premute_volume } else { 0.7 };
        guard.muted = false;
        (restored, state.runtime.core().set_volume(restored))
    } else {
        // Already in the requested state — a no-op that still reports the
        // current nominal level.
        let nominal = if guard.muted { guard.premute_volume } else { live };
        (nominal, Ok(()))
    };
    let muted_now = guard.muted;
    drop(guard);

    if let Err(e) = set_result {
        return runtime_error(&e.to_string());
    }
    json(200, serde_json::json!({"volume": canon_volume(nominal), "muted": muted_now}))
}

/// The NOMINAL volume (what `now`/`volume`/`mute` all report): the live
/// player volume, EXCEPT while muted, where it's the stashed `premute_volume`
/// — the player's real output is 0.0 while muted, but the reported level
/// stays at what the user set it to, so `vol 80%` keeps reading `80%` through
/// a mute/unmute cycle. Returns `(muted, nominal)`.
pub(super) fn nominal_volume(state: &ApiState, live: f32) -> (bool, f32) {
    match state.shared.lock() {
        Ok(s) => (s.muted, if s.muted { s.premute_volume } else { live }),
        Err(_) => (false, live),
    }
}
