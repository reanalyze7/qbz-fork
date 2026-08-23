use serde_json::Value;

use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

use super::format::{fmt_khz, fmt_mmss};
use super::volume::fraction_to_pct;

// ============================ `now` ============================

/// `qbzd now [--json]` — renders `GET /api/now-playing` (§2.2/§3.3.4).
/// Exit: 0 · 3 · 4 (via `ApiClient`/`error_from_envelope`, unchanged here).
pub async fn now(host: Option<String>, json: bool, roots: &ProfileRoots) -> i32 {
    let client = ApiClient::new(host, roots);
    match client.get("/api/now-playing").await {
        Ok(v) => {
            if json {
                println!("{}", serde_json::to_string(&v).unwrap_or_default());
            } else {
                println!("{}", render_now(&v));
            }
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}

/// `playing · Chick Corea – Spain · 3:12/9:41 · 96kHz/24bit · vol 80%` /
/// `stopped · queue 14 tracks` (02 §2.2, verbatim shape).
pub(super) fn render_now(v: &Value) -> String {
    let track = v.get("track").filter(|t| !t.is_null());
    let Some(track) = track else {
        let queue_len = v.pointer("/playback/queue_len").and_then(|n| n.as_u64()).unwrap_or(0);
        return format!("stopped · queue {queue_len} tracks");
    };

    let is_playing = v.pointer("/playback/is_playing").and_then(|b| b.as_bool()).unwrap_or(false);
    let state = if is_playing { "playing" } else { "paused" };
    let artist = track.get("artist").and_then(|a| a.as_str()).unwrap_or("");
    let title = track.get("title").and_then(|a| a.as_str()).unwrap_or("");
    let pos = v.pointer("/playback/position").and_then(|p| p.as_u64()).unwrap_or(0);
    let dur = v.pointer("/playback/duration").and_then(|p| p.as_u64()).unwrap_or(0);
    let vol = v.pointer("/playback/volume").and_then(|p| p.as_f64()).unwrap_or(0.0);
    let sr = v.pointer("/playback/sample_rate").and_then(|p| p.as_u64());
    let bd = v.pointer("/playback/bit_depth").and_then(|p| p.as_u64());

    let mut parts = vec![
        state.to_string(),
        format!("{artist} – {title}"),
        format!("{}/{}", fmt_mmss(pos), fmt_mmss(dur)),
    ];
    if let (Some(sr), Some(bd)) = (sr, bd) {
        parts.push(format!("{}/{bd}bit", fmt_khz(sr)));
    }
    parts.push(format!("vol {}%", fraction_to_pct(vol)));
    parts.join(" · ")
}
