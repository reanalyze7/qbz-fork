// crates/qbzd/src/cli/status/ — the `status` and `ping` verbs (02 §2.2).
//
// Both render an already-parsed API payload; neither holds state. `status` also
// runs the version-skew check (§1.6, from the /api/status payload — it carries
// `version` + `api_version`, so it needs no /api/info fallback) and, on the
// daemon box, the linger check (§1.4). Exit codes come from the frozen table
// (§1.3): 0 healthy · 3 unreachable · 4 needs_auth · 5 device unopenable.
mod exit_code;
mod fmt;
mod linger;
mod render;
#[cfg(test)]
mod tests;

use crate::cli::client::ApiClient;
use crate::cli::copy;
use crate::paths::ProfileRoots;

use exit_code::exit_from_state;
use linger::linger_warning;
use render::render;

/// `qbzd ping` — liveness. Human `pong`; `--json` the raw body. Exit 0 · 3.
pub async fn ping(host: Option<String>, json: bool, roots: &ProfileRoots) -> i32 {
    let client = ApiClient::new(host, roots);
    match client.get("/api/ping").await {
        Ok(v) => {
            if json {
                println!("{}", serde_json::to_string(&v).unwrap_or_default());
            } else {
                println!("pong");
            }
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}

/// `qbzd status` — THE diagnostic. Human composite block; `--json` raw payload.
/// Exit 0 healthy · 3 unreachable · 4 needs_auth · 5 device unopenable.
pub async fn status(host: Option<String>, json: bool, roots: &ProfileRoots) -> i32 {
    let client = ApiClient::new(host, roots);
    let payload = match client.get("/api/status").await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return e.exit_code();
        }
    };

    // Version skew (§1.6): breaking api_version mismatch refuses; a semver-only
    // mismatch is a warning that does not stop the render.
    let daemon_api = payload.get("api_version").and_then(|a| a.as_u64()).unwrap_or(0) as u32;
    if daemon_api != crate::API_VERSION {
        eprintln!("{}", copy::api_version_skew(daemon_api, crate::API_VERSION));
        return 1;
    }
    let cli_ver = env!("CARGO_PKG_VERSION");
    if let Some(daemon_ver) = payload.get("version").and_then(|v| v.as_str()) {
        if !daemon_ver.is_empty() && daemon_ver != cli_ver {
            eprintln!("{}", copy::version_skew(daemon_ver, cli_ver));
        }
    }

    if json {
        println!("{}", serde_json::to_string(&payload).unwrap_or_default());
    } else {
        print!("{}", render(&payload, client.host()));
    }

    // Linger check on the daemon box only (§1.4) — a warning, never fatal.
    if client.is_local() {
        if let Some(w) = linger_warning() {
            eprintln!("{w}");
        }
    }

    exit_from_state(&payload)
}
