use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{err_json, json, ApiState};

use super::internal::auth_gate;

/// `POST /api/playlist/create`. Body `{"name": "...", "description"?: "...",
/// "public"?: bool}`. Returns the created playlist.
pub fn create(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let name = match body.get("name").and_then(|v| v.as_str()).filter(|n| !n.trim().is_empty()) {
        Some(n) => n,
        None => return err_json(400, "bad_request", "create requires a name", "body: {\"name\": \"My Playlist\"}"),
    };
    let desc = body.get("description").and_then(|v| v.as_str());
    let public = body.get("public").and_then(|v| v.as_bool()).unwrap_or(false);
    match state.rt.block_on(state.runtime.core().create_playlist(name, desc, public)) {
        Ok(pl) => json(200, serde_json::json!({"playlist": serde_json::to_value(pl).unwrap_or(Value::Null)})),
        Err(_) => err_json(502, "playlists_failed", "playlist create failed", "try again in a moment"),
    }
}

/// `POST /api/playlist/update`. Body `{"id": N, "name"?, "description"?,
/// "public"?}`. Only the present fields change.
pub fn update(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let id = match body.get("id").and_then(|v| v.as_u64()) {
        Some(id) => id,
        None => return err_json(400, "bad_request", "update requires an id", "body: {\"id\": 987, \"name\": \"...\"}"),
    };
    let name = body.get("name").and_then(|v| v.as_str());
    let desc = body.get("description").and_then(|v| v.as_str());
    let public = body.get("public").and_then(|v| v.as_bool());
    match state.rt.block_on(state.runtime.core().update_playlist(id, name, desc, public)) {
        Ok(pl) => json(200, serde_json::json!({"playlist": serde_json::to_value(pl).unwrap_or(Value::Null)})),
        Err(_) => err_json(502, "playlists_failed", "playlist update failed", "try again in a moment"),
    }
}

/// `POST /api/playlist/delete`. Body `{"id": N}`. Deletes an owned playlist.
pub fn delete(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let id = match body.get("id").and_then(|v| v.as_u64()) {
        Some(id) => id,
        None => return err_json(400, "bad_request", "delete requires an id", "body: {\"id\": 987}"),
    };
    match state.rt.block_on(state.runtime.core().delete_playlist(id)) {
        Ok(()) => json(200, serde_json::json!({"ok": true, "deleted": id})),
        Err(_) => err_json(502, "playlists_failed", "playlist delete failed", "try again in a moment"),
    }
}
