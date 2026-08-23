use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

// ============================ play/pause/toggle/stop ============================

/// `POST` to `path` with no body, printing the response's `state` field
/// (`play`/`pause`/`toggle`/`stop` share this exact shape — 02 §3.3.5-8).
async fn transport_state(host: Option<String>, roots: &ProfileRoots, path: &str) -> i32 {
    let client = ApiClient::new(host, roots);
    match client.post(path, serde_json::json!({})).await {
        Ok(v) => {
            let state = v.get("state").and_then(|s| s.as_str()).unwrap_or("");
            println!("{state}");
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}

pub async fn play(host: Option<String>, roots: &ProfileRoots) -> i32 {
    transport_state(host, roots, "/api/playback/play").await
}

pub async fn pause(host: Option<String>, roots: &ProfileRoots) -> i32 {
    transport_state(host, roots, "/api/playback/pause").await
}

pub async fn toggle(host: Option<String>, roots: &ProfileRoots) -> i32 {
    transport_state(host, roots, "/api/playback/toggle").await
}

pub async fn stop(host: Option<String>, roots: &ProfileRoots) -> i32 {
    transport_state(host, roots, "/api/playback/stop").await
}
