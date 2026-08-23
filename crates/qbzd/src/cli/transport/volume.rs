use serde_json::Value;

use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

// ============================ volume ============================

/// A parsed `qbzd volume` argument (02 §2.2: 0-100 absolute, or `+N`/`-N`
/// relative — both in CLI percent-space).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeArg {
    Absolute(u8),
    Delta(i32),
}

/// `80` -> absolute 0-100 · `+5`/`-5` -> relative percent.
pub fn parse_volume_arg(s: &str) -> Result<VolumeArg, String> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix('+') {
        return rest
            .parse::<i32>()
            .map(VolumeArg::Delta)
            .map_err(|_| format!("invalid volume offset '{s}' — expected +N"));
    }
    if let Some(rest) = s.strip_prefix('-') {
        return rest
            .parse::<i32>()
            .map(|n| VolumeArg::Delta(-n))
            .map_err(|_| format!("invalid volume offset '{s}' — expected -N"));
    }
    let n: u8 = s
        .parse()
        .map_err(|_| format!("invalid volume '{s}' — expected 0-100, +N, or -N"))?;
    if n > 100 {
        return Err(format!("invalid volume '{s}' — must be 0-100"));
    }
    Ok(VolumeArg::Absolute(n))
}

/// CLI 0-100 <-> API 0.0-1.0 (02 §2.2: "CLI speaks 0-100; the API speaks
/// 0.0-1.0 — legacy contract"). `f64` throughout — JSON numbers ARE f64
/// (`serde_json::Number`), and computing in f32 then widening to build the
/// request body round-trips imprecisely (`0.8f32 as f64` != the `0.8` JSON
/// literal); the eventual `as f32` narrowing happens once, server-side,
/// right before the `Player::set_volume` call.
pub fn pct_to_fraction(pct: u8) -> f64 {
    (pct as f64 / 100.0).clamp(0.0, 1.0)
}

pub fn fraction_to_pct(frac: f64) -> i64 {
    (frac.clamp(0.0, 1.0) * 100.0).round() as i64
}

/// `VolumeArg` -> the `POST /api/playback/volume` body: `{"volume"}`
/// (absolute fraction, legacy field name) or `{"delta"}` (additive fraction —
/// the CLI's `+N`/`-N` percent converted to the API's 0.0-1.0 space).
pub fn volume_body(arg: VolumeArg) -> Value {
    match arg {
        VolumeArg::Absolute(pct) => serde_json::json!({"volume": pct_to_fraction(pct)}),
        VolumeArg::Delta(pct) => serde_json::json!({"delta": pct as f64 / 100.0}),
    }
}

/// `qbzd volume [<0-100>|+N|-N] [--json]`. Bare = read via `GET
/// /api/now-playing`, extracting `{volume, muted}` (no dedicated read route,
/// 02 §2.2). With an argument: `POST /api/playback/volume`. Exit:
/// 0 · 1 · 2 (local parse failure) · 3 · 5.
pub async fn volume(host: Option<String>, roots: &ProfileRoots, value: Option<String>, json: bool) -> i32 {
    let client = ApiClient::new(host, roots);
    match value {
        None => match client.get("/api/now-playing").await {
            Ok(v) => {
                let vol = v.pointer("/playback/volume").and_then(|x| x.as_f64()).unwrap_or(0.0);
                let muted = v.pointer("/playback/muted").and_then(|x| x.as_bool()).unwrap_or(false);
                if json {
                    let out = serde_json::json!({"volume": vol, "muted": muted});
                    println!("{}", serde_json::to_string(&out).unwrap_or_default());
                } else {
                    let pct = fraction_to_pct(vol);
                    if muted {
                        println!("vol {pct}% (muted)");
                    } else {
                        println!("vol {pct}%");
                    }
                }
                0
            }
            Err(e) => {
                eprintln!("{e}");
                e.exit_code()
            }
        },
        Some(arg) => {
            let parsed = match parse_volume_arg(&arg) {
                Ok(a) => a,
                Err(msg) => {
                    eprintln!("error: {msg}");
                    return 2;
                }
            };
            match client.post("/api/playback/volume", volume_body(parsed)).await {
                Ok(v) => {
                    let vol = v.get("volume").and_then(|x| x.as_f64()).unwrap_or(0.0);
                    println!("vol {}%", fraction_to_pct(vol));
                    0
                }
                Err(e) => {
                    eprintln!("{e}");
                    e.exit_code()
                }
            }
        }
    }
}
