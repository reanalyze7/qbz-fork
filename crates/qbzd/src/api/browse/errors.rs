use std::io::Cursor;

use tiny_http::Response;

use crate::api::{err_json, ApiState};
use crate::state::AuthState;

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

pub(super) fn not_found(kind: &str, id: &str) -> Response<Cursor<Vec<u8>>> {
    err_json(404, "not_found", &format!("{kind} {id} not found"), "check the id: qbzd search <QUERY>")
}
