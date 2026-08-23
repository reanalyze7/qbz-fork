use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

use super::cli_index_to_api;

// ============================ move / jump / stop-after ============================

/// `qbzd queue move <FROM> <TO>` -> `POST /api/queue/move`. FROM/TO are 1-based
/// positions (the same convention as `queue remove`), translated to 0-based at
/// this boundary. Exit: 0 · 1 · 2 (position 0) · 3 · 6 (out of range).
pub async fn move_(host: Option<String>, roots: &ProfileRoots, from: usize, to: usize) -> i32 {
    let from_i = match cli_index_to_api(from) {
        Ok(i) => i,
        Err(msg) => {
            eprintln!("error: {msg}");
            return 2;
        }
    };
    let to_i = match cli_index_to_api(to) {
        Ok(i) => i,
        Err(msg) => {
            eprintln!("error: {msg}");
            return 2;
        }
    };
    let client = ApiClient::new(host, roots);
    match client.post("/api/queue/move", serde_json::json!({"from": from_i, "to": to_i})).await {
        Ok(_) => {
            println!("moved {from} -> {to}");
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}

/// `qbzd queue jump <POS>` -> `POST /api/queue/jump`. POS is 1-based; jumping
/// starts audio (a click-to-play-row). Exit: 0 · 1 · 2 · 3 · 4 · 6.
pub async fn jump(host: Option<String>, roots: &ProfileRoots, position: usize) -> i32 {
    let index = match cli_index_to_api(position) {
        Ok(i) => i,
        Err(msg) => {
            eprintln!("error: {msg}");
            return 2;
        }
    };
    let client = ApiClient::new(host, roots);
    match client.post("/api/queue/jump", serde_json::json!({"index": index})).await {
        Ok(v) => {
            let title = v.get("track").and_then(|t| t.get("title")).and_then(|x| x.as_str()).unwrap_or("");
            let artist = v.get("track").and_then(|t| t.get("artist")).and_then(|x| x.as_str()).unwrap_or("");
            if title.is_empty() {
                println!("jumped to {position}");
            } else {
                println!("jumped: {position}  \"{title}\" — {artist}");
            }
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}

/// `qbzd queue stop-after [current|off]` (bare = current) -> `POST
/// /api/queue/stop-after`. Stops playback after the current track, or clears
/// the gate. Exit: 0 · 1 · 2 · 3.
pub async fn stop_after(host: Option<String>, roots: &ProfileRoots, arg: Option<String>) -> i32 {
    let body = match arg.as_deref() {
        None | Some("current") => serde_json::json!({"current": true}),
        Some("off") => serde_json::json!({"off": true}),
        Some(other) => {
            eprintln!("error: unknown argument '{other}'");
            eprintln!("  → usage: qbzd queue stop-after [current|off]");
            return 2;
        }
    };
    let client = ApiClient::new(host, roots);
    match client.post("/api/queue/stop-after", body).await {
        Ok(v) => {
            match v.get("stop_after_track_id").and_then(|x| x.as_u64()) {
                Some(id) => println!("stop-after set (track {id})"),
                None => println!("stop-after cleared"),
            }
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}
