// crates/qbzd/src/api/fav/ — favorites (02 §2.3, §3.4 rows 20-22): GET
// /api/favorites (paged read), POST /api/favorites/add|remove.
//
// The plural-read/singular-write fav_type trap (get_favorites wants "tracks",
// add_favorite wants "track" — qbz-core/src/core.rs:1072 vs :1088) is HIDDEN
// from clients: the contract is SINGULAR everywhere (track|album|artist) and
// the server pluralizes for the read. All auth-gated (favorites are per-user).
mod fav_type;
mod query;

use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::state::AuthState;

use super::{err_json, json, ApiState};
use fav_type::{bad_type, FavType};
use query::pairs;

const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 500;

/// `GET /api/favorites?type=track|album|artist&limit=&offset=`. Returns the
/// raw favorites payload under `favorites` (its `items` arrays are the same
/// shape the CLI's generic renderer walks). `type` defaults to `track`.
pub fn list(state: &ApiState, query: &str) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let mut ftype = FavType::Track;
    let mut limit = DEFAULT_LIMIT;
    let mut offset = 0u32;
    for (k, v) in pairs(query) {
        match k.as_str() {
            "type" | "fav_type" => match FavType::parse(&v) {
                Some(t) => ftype = t,
                None => return bad_type(&v),
            },
            "limit" => {
                if let Ok(n) = v.parse::<u32>() {
                    limit = n.clamp(1, MAX_LIMIT);
                }
            }
            "offset" => {
                if let Ok(n) = v.parse::<u32>() {
                    offset = n;
                }
            }
            _ => {}
        }
    }

    match state.rt.block_on(state.runtime.core().get_favorites(ftype.plural(), limit, offset)) {
        Ok(v) => json(200, serde_json::json!({"type": ftype.singular(), "favorites": v})),
        Err(_) => err_json(502, "favorites_failed", "favorites request to Qobuz failed", "try again in a moment"),
    }
}

/// `POST /api/favorites/add`. Body `{"fav_type": "track|album|artist",
/// "item_id": "..."}` or `{"fav_type": "track", "current": true}` (favorite the
/// now-playing track). Idempotent from the caller's view (Qobuz de-dupes).
pub fn add(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    mutate(state, body, true)
}

/// `POST /api/favorites/remove`. Same body shape (minus `current`).
pub fn remove(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    mutate(state, body, false)
}

fn mutate(state: &ApiState, body: &Value, adding: bool) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let ftype = match body.get("fav_type").and_then(|v| v.as_str()).and_then(FavType::parse) {
        Some(t) => t,
        None => return err_json(400, "bad_request", "requires fav_type: track|album|artist", "body: {\"fav_type\":\"track\",\"item_id\":\"...\"}"),
    };

    // `current: true` (add only) favorites the now-playing track.
    let item_id = if adding && body.get("current").and_then(|v| v.as_bool()).unwrap_or(false) {
        if ftype != FavType::Track {
            return err_json(400, "bad_request", "current only applies to fav_type track", "use: fav add track --current");
        }
        match state.rt.block_on(state.runtime.core().get_queue_state()).current_track {
            Some(t) => t.id.to_string(),
            None => return err_json(404, "not_found", "nothing is playing", "queue a track first"),
        }
    } else {
        match body.get("item_id").and_then(|v| v.as_str()) {
            Some(id) if !id.is_empty() => id.to_string(),
            _ => return err_json(400, "bad_request", "requires an item_id", "body: {\"fav_type\":\"track\",\"item_id\":\"176544871\"}"),
        }
    };

    let result = if adding {
        state.rt.block_on(state.runtime.core().add_favorite(ftype.singular(), &item_id))
    } else {
        state.rt.block_on(state.runtime.core().remove_favorite(ftype.singular(), &item_id))
    };
    match result {
        Ok(()) => json(200, serde_json::json!({"ok": true, "type": ftype.singular(), "item_id": item_id})),
        Err(_) => err_json(502, "favorites_failed", "favorites update failed", "try again in a moment"),
    }
}

fn auth_gate(state: &ApiState) -> Option<Response<Cursor<Vec<u8>>>> {
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
