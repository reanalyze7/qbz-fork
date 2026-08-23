use std::io::Cursor;

use tiny_http::Response;

use crate::state::AuthState;

use super::super::{err_json, ApiState};

/// 409 `needs_auth` — search needs a live Qobuz session. Mirrors
/// `queue::add`'s gate (this file's self-contained-helpers convention).
pub(super) fn auth_gate(state: &ApiState) -> Option<Response<Cursor<Vec<u8>>>> {
    let needs_auth = state
        .shared
        .lock()
        .map(|s| s.auth == AuthState::NeedsAuth)
        .unwrap_or(false);
    if needs_auth {
        Some(err_json(
            409,
            "needs_auth",
            "not logged in to Qobuz",
            "run: qbzd login",
        ))
    } else {
        None
    }
}

/// 502 for any upstream Qobuz/core failure. The code maps to CLI exit 1
/// (`error_from_envelope`'s catch-all), never a panic. The `CoreError` is not
/// interpolated (matches `queue::add`'s `Err(_)` discipline — the daemon does
/// not leak upstream error text through the control plane).
pub(super) fn upstream_error() -> Response<Cursor<Vec<u8>>> {
    err_json(
        502,
        "search_failed",
        "search request to Qobuz failed",
        "try again in a moment; check: qbzd status",
    )
}
