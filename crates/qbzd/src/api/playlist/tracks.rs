use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{err_json, json, ApiState};

use super::internal::{auth_gate, parse_ids};

/// `POST /api/playlist/tracks/add`. Body `{"id": N, "track_ids": [...]}`.
pub fn tracks_add(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let id = match body.get("id").and_then(|v| v.as_u64()) {
        Some(id) => id,
        None => return err_json(400, "bad_request", "add requires a playlist id", "body: {\"id\": 987, \"track_ids\": [...]}"),
    };
    let track_ids = match parse_ids(body) {
        Ok(ids) => ids,
        Err((m, h)) => return err_json(400, "bad_request", &m, &h),
    };
    match state.rt.block_on(state.runtime.core().add_tracks_to_playlist(id, &track_ids)) {
        Ok(()) => json(200, serde_json::json!({"ok": true, "added": track_ids.len()})),
        Err(_) => err_json(502, "playlists_failed", "add to playlist failed", "try again in a moment"),
    }
}

/// `POST /api/playlist/tracks/remove`. Body `{"id": N, "track_ids": [...]}` —
/// PLAIN track ids. The daemon resolves them to per-playlist `playlist_track_id`
/// row ids (the row-id trap, qbz-models Track.playlist_track_id) before calling
/// `remove_tracks_from_playlist`, so clients never touch row ids.
pub fn tracks_remove(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let id = match body.get("id").and_then(|v| v.as_u64()) {
        Some(id) => id,
        None => return err_json(400, "bad_request", "remove requires a playlist id", "body: {\"id\": 987, \"track_ids\": [...]}"),
    };
    let track_ids = match parse_ids(body) {
        Ok(ids) => ids,
        Err((m, h)) => return err_json(400, "bad_request", &m, &h),
    };
    let pl = match state.rt.block_on(state.runtime.core().get_playlist(id)) {
        Ok(p) => p,
        Err(_) => return err_json(404, "not_found", &format!("playlist {id} not found"), "check: qbzd playlist list"),
    };
    let wanted: std::collections::HashSet<u64> = track_ids.iter().copied().collect();
    let items = pl.tracks.map(|t| t.items).unwrap_or_default();
    let row_ids: Vec<u64> = items
        .iter()
        .filter(|t| wanted.contains(&t.id))
        .filter_map(|t| t.playlist_track_id)
        .collect();
    if row_ids.is_empty() {
        return err_json(404, "not_found", "none of those tracks are in the playlist", "check: qbzd playlist show <ID>");
    }
    match state.rt.block_on(state.runtime.core().remove_tracks_from_playlist(id, &row_ids)) {
        Ok(()) => json(200, serde_json::json!({"ok": true, "removed": row_ids.len()})),
        Err(_) => err_json(502, "playlists_failed", "remove from playlist failed", "try again in a moment"),
    }
}
