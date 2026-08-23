use std::io::Cursor;

use tiny_http::Response;

use crate::api::{err_json, ApiState};
use crate::state::AuthState;

/// 409 `needs_auth` (01 §6.2 / 02 §3.1.3 example envelope, verbatim).
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

/// 503 `audio_unavailable` — the frozen taxonomy's device/audio bucket
/// (02 §3.1.3), exit 5. Reserved for GENUINE audio/device conditions: the
/// DSD-direct guards (handled inline via `err_json`, not this helper) and
/// cold-start's `play_track_resolved` failure (no device / stream resolve
/// failed). Each route's documented exit set (02 §2.2) decides which one
/// applies — `pause`/`stop`/plain `seek`/`volume`/`next`/`prev` never list
/// exit 5, so their `Player`/`QbzCore` command failures use
/// [`runtime_error`] instead.
#[allow(dead_code)]
pub(super) fn device_error(message: &str) -> Response<Cursor<Vec<u8>>> {
    err_json(503, "audio_unavailable", message, "check: qbzd status")
}

/// A generic runtime failure, exit 1 (02 §1.3's catch-all) — e.g. the
/// player's command channel is dead. `code` "internal" is NOT one of
/// `error_from_envelope`'s special-cased codes, so it falls to
/// `CliError::Runtime` client-side.
pub(super) fn runtime_error(message: &str) -> Response<Cursor<Vec<u8>>> {
    err_json(500, "internal", message, "check: qbzd status")
}
