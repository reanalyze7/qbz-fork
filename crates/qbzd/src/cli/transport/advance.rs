use serde_json::Value;

use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

// ============================ next/prev ============================

/// `POST` to `path` with no body; the response is the landing `QueueTrack` or
/// `null` at queue end (02 §3.3.9-10, legacy shape).
async fn transport_advance(host: Option<String>, roots: &ProfileRoots, path: &str) -> i32 {
    let client = ApiClient::new(host, roots);
    match client.post(path, serde_json::json!({})).await {
        Ok(v) => {
            println!("{}", render_advance(&v));
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}

/// `-> Chick Corea – 500 Miles High` / `queue finished` (02 §2.2, verbatim) —
/// or `-> queued (next)` for the spawn-and-ack advance response (the ritual
/// runs async; follow the landing track via `qbzd now`).
pub(super) fn render_advance(v: &Value) -> String {
    if v.is_null() {
        return "queue finished".to_string();
    }
    if v.get("queued").and_then(|q| q.as_bool()).unwrap_or(false) {
        let direction = v.get("direction").and_then(|d| d.as_str()).unwrap_or("next");
        return format!("-> queued ({direction})");
    }
    let artist = v.get("artist").and_then(|s| s.as_str()).unwrap_or("");
    let title = v.get("title").and_then(|s| s.as_str()).unwrap_or("");
    format!("-> {artist} – {title}")
}

pub async fn next(host: Option<String>, roots: &ProfileRoots) -> i32 {
    transport_advance(host, roots, "/api/playback/next").await
}

pub async fn prev(host: Option<String>, roots: &ProfileRoots) -> i32 {
    transport_advance(host, roots, "/api/playback/previous").await
}
