use serde_json::Value;

use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

use super::cli_index_to_api;

// ============================ add ============================

/// `qbzd queue add <TRACK_ID> [--next]` -> `POST /api/queue/add` (§2.2/
/// §3.3.14). Exit: 0 · 1 · 3 · 4 (needs_auth) · 6 (unknown id).
pub async fn add(host: Option<String>, roots: &ProfileRoots, track_id: u64, next: bool) -> i32 {
    let client = ApiClient::new(host, roots);
    let position = if next { "next" } else { "end" };
    let body = serde_json::json!({"track_ids": [track_id], "position": position});
    match client.post("/api/queue/add", body).await {
        Ok(v) => {
            println!("{}", render_added(&v, next));
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}

/// `added: Spain – Chick Corea (next)` (§2.2, verbatim). The daemon
/// additively returns the materialized `tracks` (api/queue.rs's `add` doc
/// comment explains why: the documented `{"added","total_tracks"}` sketch
/// alone has no title/artist, and §1.1 forbids a second request to fetch
/// them). A response without `tracks` (an older daemon, or the batch path
/// with `track_ids.len() > 1`, which this CLI never sends) falls back to the
/// bare count.
pub(super) fn render_added(v: &Value, next: bool) -> String {
    let suffix = if next { " (next)" } else { "" };
    let track = v
        .get("tracks")
        .and_then(|t| t.as_array())
        .filter(|a| a.len() == 1)
        .and_then(|a| a.first());
    match track {
        Some(t) => {
            let title = t.get("title").and_then(|x| x.as_str()).unwrap_or("");
            let artist = t.get("artist").and_then(|x| x.as_str()).unwrap_or("");
            format!("added: {title} – {artist}{suffix}")
        }
        None => {
            let added = v.get("added").and_then(|a| a.as_u64()).unwrap_or(0);
            format!("added: {added} track(s){suffix}")
        }
    }
}

// ============================ remove ============================

/// `qbzd queue remove <INDEX>` -> `POST /api/queue/remove` (§2.2/§3.3.15).
/// `index` is the 1-based position `queue list` displayed; translated to the
/// API's 0-based index at this boundary. Exit: 0 · 1 (removing the playing
/// track — server `bad_request`, not one of `error_from_envelope`'s
/// special-cased codes, §3.1.3) · 2 (position 0, local usage error) · 3 ·
/// 6 (out of range — server `not_found`).
pub async fn remove(host: Option<String>, roots: &ProfileRoots, index: usize) -> i32 {
    let api_index = match cli_index_to_api(index) {
        Ok(i) => i,
        Err(msg) => {
            eprintln!("error: {msg}");
            return 2;
        }
    };
    let client = ApiClient::new(host, roots);
    let body = serde_json::json!({"index": api_index});
    match client.post("/api/queue/remove", body).await {
        Ok(v) => {
            println!("{}", render_removed(&v));
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}

pub(super) fn render_removed(v: &Value) -> String {
    let id = v.get("removed").and_then(|r| r.as_u64()).unwrap_or(0);
    let total = v.get("total_tracks").and_then(|t| t.as_u64()).unwrap_or(0);
    format!("removed: track {id} · {total} left")
}

// ============================ clear ============================

/// `qbzd queue clear [--keep-current]` -> `POST /api/queue/clear` (§2.2/
/// §3.3.16). The flag is a plain bool (clap default `false` when absent):
/// bare `queue clear` sends `keep_current: false` (a full reset — the
/// harshest reading of "clear"); `--keep-current` sends `true` (preserve the
/// now-playing track). The API's own default (`true` when the field is
/// OMITTED, §3.3.16) never applies here — this CLI always sends the field
/// explicitly. Exit: 0 · 1 · 3.
pub async fn clear(host: Option<String>, roots: &ProfileRoots, keep_current: bool) -> i32 {
    let client = ApiClient::new(host, roots);
    let body = serde_json::json!({"keep_current": keep_current});
    match client.post("/api/queue/clear", body).await {
        Ok(v) => {
            let total = v.get("total_tracks").and_then(|t| t.as_u64()).unwrap_or(0);
            println!("queue cleared · {total} left");
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}
