use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{err_json, json, ApiState};

use super::errors::{auth_gate, not_found};
use super::query::{limit_offset, parse};

/// `GET /api/artist?id=<ID>&view=page|top|albums&limit=&offset=&release_type=`.
/// `page` (default) = the artist page (bio, top tracks, similar); `top` = the
/// full top-tracks list; `albums` = the paged releases grid.
pub fn artist(state: &ApiState, query: &str) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let p = parse(query);
    let id = match p.get("id").and_then(|v| v.parse::<u64>().ok()) {
        Some(id) => id,
        None => return err_json(400, "bad_request", "artist requires a numeric id", "usage: qbzd artist <ARTIST_ID>"),
    };
    let (limit, offset) = limit_offset(&p);
    let view = p.get("view").map(String::as_str).unwrap_or("page");

    let core = state.runtime.core();
    match view {
        "page" => match state.rt.block_on(core.get_artist_page(id, None)) {
            Ok(r) => json(200, serde_json::json!({"view": "page", "page": serde_json::to_value(r).unwrap_or(Value::Null)})),
            Err(_) => not_found("artist", &id.to_string()),
        },
        "top" => match state.rt.block_on(core.get_artist_tracks(id, limit, offset)) {
            Ok(tc) => json(200, serde_json::json!({"view": "top", "tracks": serde_json::to_value(tc).unwrap_or(Value::Null)})),
            Err(_) => not_found("artist", &id.to_string()),
        },
        "albums" => {
            let release_type = p.get("release_type").map(String::as_str).unwrap_or("album");
            match state.rt.block_on(core.get_releases_grid(id, release_type, limit, offset, None)) {
                Ok(r) => json(200, serde_json::json!({"view": "albums", "releases": serde_json::to_value(r).unwrap_or(Value::Null)})),
                Err(_) => not_found("artist", &id.to_string()),
            }
        }
        other => err_json(400, "bad_request", &format!("unknown view '{other}'"), "view: page | top | albums"),
    }
}
