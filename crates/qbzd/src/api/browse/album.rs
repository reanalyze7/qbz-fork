use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{json, ApiState};

use super::errors::{auth_gate, not_found};
use crate::api::err_json;
use super::query::{parse, wants};

/// `GET /api/album?id=<ALBUM_ID>&suggest=<0|1>`. The full album envelope
/// (tracklist, ImageSet artwork, description, awards) via `core.get_album`;
/// `suggest=1` also includes similar albums (`get_album_suggest`).
pub fn album(state: &ApiState, query: &str) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let p = parse(query);
    let id = match p.get("id") {
        Some(v) if !v.is_empty() => v.clone(),
        _ => return err_json(400, "bad_request", "album requires an id", "usage: qbzd album <ALBUM_ID>"),
    };

    let album = match state.rt.block_on(state.runtime.core().get_album(&id)) {
        Ok(a) => serde_json::to_value(a).unwrap_or(Value::Null),
        Err(_) => return not_found("album", &id),
    };
    let similar = if wants(&p, "suggest") {
        match state.rt.block_on(state.runtime.core().get_album_suggest(&id)) {
            Ok(s) => serde_json::to_value(s).unwrap_or(Value::Null),
            Err(_) => Value::Null,
        }
    } else {
        Value::Null
    };

    json(200, serde_json::json!({"album": album, "similar": similar}))
}
