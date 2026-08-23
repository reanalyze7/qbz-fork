use serde_json::Value;

use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

use super::volume::fraction_to_pct;

// ============================ mute ============================

/// `None` (bare = toggle) / `Some("on")` / `Some("off")` -> the
/// `{"mute": "on"|"off"|"toggle"}` body form (02 §2.2/§3.3.12 — "same route
/// as volume, costs no extra route").
pub fn mute_body(arg: Option<&str>) -> Result<Value, String> {
    match arg {
        None => Ok(serde_json::json!({"mute": "toggle"})),
        Some("on") => Ok(serde_json::json!({"mute": "on"})),
        Some("off") => Ok(serde_json::json!({"mute": "off"})),
        Some(other) => Err(format!("invalid mute state '{other}' — use on or off")),
    }
}

/// `qbzd mute [on|off]` — human `muted (was 80%)` / `unmuted · vol 80%`
/// (02 §2.2, verbatim). Exit: 0 · 1 · 2 (local parse failure) · 3 · 5.
pub async fn mute(host: Option<String>, roots: &ProfileRoots, state_arg: Option<String>) -> i32 {
    let body = match mute_body(state_arg.as_deref()) {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("error: {msg}");
            return 2;
        }
    };
    let client = ApiClient::new(host, roots);
    match client.post("/api/playback/volume", body).await {
        Ok(v) => {
            let vol = v.get("volume").and_then(|x| x.as_f64()).unwrap_or(0.0);
            let muted = v.get("muted").and_then(|x| x.as_bool()).unwrap_or(false);
            let pct = fraction_to_pct(vol);
            if muted {
                println!("muted (was {pct}%)");
            } else {
                println!("unmuted · vol {pct}%");
            }
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}
