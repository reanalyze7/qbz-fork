use serde_json::Value;

use super::fmt::{fmt_mmss, fmt_uptime, str_at};

/// The §2.2 composite block. `host` is the target (the payload has no `bind`).
pub(super) fn render(p: &Value, host: &str) -> String {
    let version = str_at(p, &["version"]);
    let api = p.get("api_version").and_then(|a| a.as_u64()).unwrap_or(0);
    let uptime = fmt_uptime(p.get("uptime_secs").and_then(|u| u.as_u64()).unwrap_or(0));
    let data_root = str_at(p, &["data_root"]);

    let mut out = String::new();
    out.push_str(&format!(
        "qbzd {version} · api v{api} · up {uptime} · {host} · data {data_root}\n"
    ));
    out.push_str(&format!("auth      : {}\n", render_auth(p)));
    out.push_str(&format!("audio     : {}\n", render_audio(p)));
    out.push_str(&format!("playback  : {}\n", render_playback(p)));
    out.push_str(&format!(
        "network   : {}\n",
        if p.pointer("/network/online").and_then(|v| v.as_bool()).unwrap_or(false) {
            "online"
        } else {
            "offline"
        }
    ));
    out.push_str(&format!("last error: {}\n", render_last_error(p)));
    out
}

fn render_auth(p: &Value) -> String {
    match str_at(p, &["auth", "state"]).as_str() {
        "logged_in" => {
            let user = p.pointer("/auth/user_id").and_then(|v| v.as_u64());
            let sub = p.pointer("/auth/subscription").and_then(|v| v.as_str());
            match (user, sub) {
                (Some(u), Some(s)) => format!("logged in (user {u}, {s})"),
                (Some(u), None) => format!("logged in (user {u})"),
                _ => "logged in".to_string(),
            }
        }
        "restoring" => "restoring session…".to_string(),
        _ => "not logged in".to_string(),
    }
}

fn render_audio(p: &Value) -> String {
    let backend = p.pointer("/audio/backend").and_then(|v| v.as_str());
    let device = p.pointer("/audio/configured_device").and_then(|v| v.as_str());
    let present = p.pointer("/audio/device_present").and_then(|v| v.as_bool()).unwrap_or(false);
    let bit_perfect = p.pointer("/audio/bit_perfect").and_then(|v| v.as_str());
    let sr = p.pointer("/audio/sample_rate").and_then(|v| v.as_u64());
    let bd = p.pointer("/audio/bit_depth").and_then(|v| v.as_u64());

    let mut parts: Vec<String> = Vec::new();
    let head = match (backend, device) {
        (Some(b), Some(d)) => format!("{b} {d}"),
        (Some(b), None) => format!("{b} (system default)"),
        (None, Some(d)) => d.to_string(),
        (None, None) => "system default".to_string(),
    };
    parts.push(head);
    parts.push(if present { "present".into() } else { "not present".into() });
    if let Some(bp) = bit_perfect {
        parts.push(format!("bit-perfect: {bp}"));
    }
    if let (Some(sr), Some(bd)) = (sr, bd) {
        parts.push(format!("{sr} Hz / {bd}-bit"));
    }
    parts.join(" · ")
}

pub(super) fn render_playback(p: &Value) -> String {
    let state = str_at(p, &["playback", "state"]);
    let queue = p.pointer("/playback/queue_len").and_then(|v| v.as_u64()).unwrap_or(0);
    if state == "stopped" {
        return format!("stopped · queue {queue}");
    }
    let title = p.pointer("/playback/title").and_then(|v| v.as_str());
    let artist = p.pointer("/playback/artist").and_then(|v| v.as_str());
    let pos = p.pointer("/playback/position").and_then(|v| v.as_u64()).unwrap_or(0);
    let dur = p.pointer("/playback/duration").and_then(|v| v.as_u64()).unwrap_or(0);
    let vol = p.pointer("/playback/volume").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let muted = p.pointer("/playback/muted").and_then(|v| v.as_bool()).unwrap_or(false);

    let track = match (title, artist) {
        (Some(t), Some(a)) => format!("\"{t}\" — {a}"),
        (Some(t), None) => format!("\"{t}\""),
        _ => "(unknown track)".to_string(),
    };
    let vol_str = if muted {
        "muted".to_string()
    } else {
        format!("vol {}%", (vol * 100.0).round() as i64)
    };
    format!(
        "{state} · {track} · {} / {} · {vol_str} · queue {queue}",
        fmt_mmss(pos),
        fmt_mmss(dur)
    )
}

fn render_last_error(p: &Value) -> String {
    for key in ["stream", "auth", "transport"] {
        if let Some(m) = p.pointer(&format!("/last_errors/{key}")).and_then(|v| v.as_str()) {
            if !m.is_empty() {
                return format!("{key}: {m}");
            }
        }
    }
    "none".to_string()
}
