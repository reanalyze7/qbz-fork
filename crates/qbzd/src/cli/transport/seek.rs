use serde_json::Value;

use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

use super::format::fmt_mmss;

// ============================ seek ============================

/// A parsed `qbzd seek` argument (02 §2.2: absolute seconds, `+N`/`-N`
/// relative, or `mm:ss`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekArg {
    Absolute(u64),
    Delta(i64),
}

/// `90` -> absolute seconds · `+30`/`-10` -> relative seconds · `1:23` ->
/// absolute seconds (mm:ss). Usage errors (exit 2) name what was expected.
pub fn parse_seek_arg(s: &str) -> Result<SeekArg, String> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix('+') {
        return rest
            .parse::<i64>()
            .map(SeekArg::Delta)
            .map_err(|_| format!("invalid seek offset '{s}' — expected +N seconds"));
    }
    if let Some(rest) = s.strip_prefix('-') {
        return rest
            .parse::<i64>()
            .map(|n| SeekArg::Delta(-n))
            .map_err(|_| format!("invalid seek offset '{s}' — expected -N seconds"));
    }
    if let Some((mm, ss)) = s.split_once(':') {
        let mm: u64 = mm
            .parse()
            .map_err(|_| format!("invalid seek position '{s}' — expected mm:ss"))?;
        let ss: u64 = ss
            .parse()
            .map_err(|_| format!("invalid seek position '{s}' — expected mm:ss"))?;
        if ss >= 60 {
            return Err(format!("invalid seek position '{s}' — seconds must be 0-59"));
        }
        return Ok(SeekArg::Absolute(mm * 60 + ss));
    }
    s.parse::<u64>()
        .map(SeekArg::Absolute)
        .map_err(|_| format!("invalid seek position '{s}' — expected seconds, +N, -N, or mm:ss"))
}

/// `SeekArg` -> the `POST /api/playback/seek` body (02 §3.3.11): `{"position"}`
/// (absolute, legacy field name) or `{"delta"}` (additive).
pub fn seek_body(arg: SeekArg) -> Value {
    match arg {
        SeekArg::Absolute(n) => serde_json::json!({"position": n}),
        SeekArg::Delta(n) => serde_json::json!({"delta": n}),
    }
}

/// `qbzd seek <POS|+N|-N|mm:ss>` — human `at 1:30 / 9:41` (02 §2.2, verbatim
/// — note the spaced slash, unlike `now`'s unspaced `3:12/9:41`). Exit:
/// 0 · 1 · 2 (local parse failure) · 3 · 5.
pub async fn seek(host: Option<String>, roots: &ProfileRoots, arg: String) -> i32 {
    let parsed = match parse_seek_arg(&arg) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("error: {msg}");
            return 2;
        }
    };
    let client = ApiClient::new(host, roots);
    match client.post("/api/playback/seek", seek_body(parsed)).await {
        Ok(v) => {
            let pos = v.get("position").and_then(|p| p.as_u64()).unwrap_or(0);
            let dur = v.get("duration").and_then(|d| d.as_u64()).unwrap_or(0);
            println!("at {} / {}", fmt_mmss(pos), fmt_mmss(dur));
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}
