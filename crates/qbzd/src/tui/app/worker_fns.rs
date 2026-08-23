// crates/qbzd/src/tui/app/worker_fns.rs — the free (non-method) worker
// functions behind status refresh, device enumeration, settings load/save,
// and the reload nudge. Kept free (not `impl App` methods) since they run
// off the main thread and only need borrowed data, not `&self`.

use qbz_audio::settings::{AudioSettings, AudioSettingsStore};
use qbz_audio::{AudioBackendType, AudioDevice, BackendManager};
use serde_json::{json, Value};

use crate::cli::client::{ApiClient, CliError};
use crate::paths::ProfileRoots;
use crate::tui::screens::network as network_screen;
use crate::tui::strings as s;

use super::worker_fns_ext::playing_extra;

pub(super) async fn fetch_status(roots: ProfileRoots) -> Option<Value> {
    let client = ApiClient::new(None, &roots);
    client.get("/api/status").await.ok()
}

pub(super) fn enumerate_devices(backend: AudioBackendType) -> Result<Vec<AudioDevice>, String> {
    BackendManager::create_backend(backend)
        .and_then(|b| b.enumerate_devices())
        .map_err(|e| e.to_string())
}

pub(super) fn load_audio(roots: &ProfileRoots) -> AudioSettings {
    AudioSettingsStore::new_at(&roots.data)
        .and_then(|s| s.get_settings())
        .unwrap_or_default()
}

/// Persist changed keys through T11's `write_one`. Returns `(Some(error_line),
/// reinit)` — a mid-set failure names the key; `reinit` is true when any written
/// key was Reinit-class (§4.3 client-side classification).
pub(super) fn write_keys(roots: &ProfileRoots, keys: &[(String, String)]) -> (Option<String>, bool) {
    let mut reinit = false;
    for (k, v) in keys {
        match crate::cli::settings::write_one(roots, k, v) {
            Ok(class) => {
                if class == crate::cli::settings::ApplyClass::Reinit {
                    reinit = true;
                }
            }
            Err(e) => {
                // The TUI only displays the message — it doesn't need the
                // Usage/Io exit-code split `settings set` maps to (see
                // `cli::settings::SetError`).
                return (Some(format!("failed to save {k}: {}", e.to_string().trim())), reinit);
            }
        }
    }
    (None, reinit)
}

pub(super) fn save_network(roots: &ProfileRoots, bind: &str, port: u16, token: Option<&str>) -> Option<String> {
    let path = roots.config.join("qbzd.toml");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    match network_screen::rewrite_toml(&existing, bind, port, token) {
        Ok(text) => match std::fs::write(&path, text) {
            Ok(()) => None,
            Err(e) => Some(format!("failed to write qbzd.toml: {e}")),
        },
        Err(e) => Some(format!("failed to rewrite qbzd.toml: {e}")),
    }
}

/// POST /api/settings/reload and compose the §4.3 result. Returns
/// `(lines, status_body, reachable)`.
pub(super) async fn do_reload(
    roots: &ProfileRoots,
    is_network: bool,
    reinit: bool,
) -> (Vec<String>, Option<Value>, bool) {
    let client = ApiClient::new(None, roots);
    match client.post("/api/settings/reload", json!({})).await {
        Ok(body) => {
            let lines = if is_network {
                vec!["saved.".to_string(), s::NETWORK_RESTART.to_string()]
            } else {
                let mut line = "saved · daemon reloaded".to_string();
                if reinit {
                    line.push_str(" (output device reinitialized");
                    if let Some(extra) = playing_extra(&body) {
                        line.push_str(&format!(" · {extra}"));
                    }
                    line.push(')');
                }
                vec![line]
            };
            (lines, Some(body), true)
        }
        Err(CliError::Unreachable(_)) => {
            let lines = if is_network {
                vec!["saved.".to_string(), s::APPLIES_ON_START.to_string()]
            } else {
                vec![s::SAVED_DISK_ONLY.to_string()]
            };
            (lines, None, false)
        }
        Err(_) => (vec![s::RELOAD_REFUSED.to_string()], None, true),
    }
}
