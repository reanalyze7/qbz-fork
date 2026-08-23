use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{err_json, json, ApiState};

use super::errors::{auth_gate, not_found};
use super::query::{limit_offset, parse};
use super::QUEUE_SEED_CAP;

/// `GET /api/similar?artist=<ID>` (similar artists) or `?album=<ID>` (similar
/// albums), `&limit=&offset=`. Exactly one selector.
pub fn similar(state: &ApiState, query: &str) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let p = parse(query);
    let (limit, offset) = limit_offset(&p);

    if let Some(artist_id) = p.get("artist").and_then(|v| v.parse::<u64>().ok()) {
        return match state.rt.block_on(state.runtime.core().get_similar_artists(artist_id, limit, offset)) {
            Ok(page) => json(200, serde_json::json!({"artists": serde_json::to_value(page).unwrap_or(Value::Null)})),
            Err(_) => not_found("artist", &artist_id.to_string()),
        };
    }
    if let Some(album_id) = p.get("album").filter(|v| !v.is_empty()) {
        return match state.rt.block_on(state.runtime.core().get_album_suggest(album_id)) {
            Ok(sug) => json(200, serde_json::json!({"albums": serde_json::to_value(sug).unwrap_or(Value::Null)})),
            Err(_) => not_found("album", album_id),
        };
    }
    err_json(400, "bad_request", "similar requires artist=<ID> or album=<ID>", "usage: qbzd similar artist:<ID> | album:<ID>")
}

/// `GET /api/suggest?seed=<ID,ID,...>&limit=`. Dynamic For-You suggestions
/// (`get_dynamic_suggest`). Seeds are explicit (`seed=`) or, when omitted,
/// the current queue's track ids (current + upcoming, capped) — so the daemon
/// tracks no listening history, honoring the UNIX-honest seeding the CONSOLE
/// brief specifies.
pub fn suggest(state: &ApiState, query: &str) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let p = parse(query);
    let (limit, _offset) = limit_offset(&p);

    let seeds: Vec<u64> = match p.get("seed") {
        Some(s) if !s.is_empty() => s.split(',').filter_map(|x| x.trim().parse::<u64>().ok()).collect(),
        _ => {
            let q = state.rt.block_on(state.runtime.core().get_queue_state());
            let mut ids: Vec<u64> = Vec::new();
            if let Some(t) = &q.current_track {
                ids.push(t.id);
            }
            for t in q.upcoming.iter().take(QUEUE_SEED_CAP.saturating_sub(1)) {
                ids.push(t.id);
            }
            ids
        }
    };
    if seeds.is_empty() {
        return err_json(
            400,
            "bad_request",
            "suggest needs seed track ids",
            "play something first, or: qbzd suggest --seed <ID,ID>",
        );
    }

    match state.rt.block_on(state.runtime.core().get_dynamic_suggest(&seeds, limit)) {
        Ok(tracks) => json(200, serde_json::json!({"tracks": serde_json::to_value(tracks).unwrap_or(Value::Null)})),
        Err(_) => err_json(502, "suggest_failed", "suggestion request to Qobuz failed", "try again in a moment"),
    }
}
