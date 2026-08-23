use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{err_json, json, ApiState};

use super::internal::{auth_gate, id_param};

/// `GET /api/playlists` — the user's playlist collection.
pub fn list(state: &ApiState) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    match state.rt.block_on(state.runtime.core().get_user_playlists()) {
        Ok(pls) => json(
            200,
            serde_json::json!({"playlists": serde_json::to_value(pls).unwrap_or(Value::Null)}),
        ),
        Err(_) => err_json(502, "playlists_failed", "playlists request to Qobuz failed", "try again in a moment"),
    }
}

/// `GET /api/playlist?id=<ID>` — one playlist with its full track list.
pub fn show(state: &ApiState, query: &str) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let id = match id_param(query) {
        Some(id) => id,
        None => return err_json(400, "bad_request", "playlist requires a numeric id", "usage: qbzd playlist show <ID>"),
    };
    match state.rt.block_on(state.runtime.core().get_playlist(id)) {
        Ok(pl) => json(
            200,
            serde_json::json!({"playlist": serde_json::to_value(pl).unwrap_or(Value::Null)}),
        ),
        Err(_) => err_json(404, "not_found", &format!("playlist {id} not found"), "check: qbzd playlist list"),
    }
}
