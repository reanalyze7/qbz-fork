// crates/qbzd/src/tui/app/worker_fns_ext.rs — small free helpers shared by
// the worker functions and the footer/import code: status-body parsing, the
// footer's pure state mapping, path expansion, and desktop-profile detection.

use qbz_audio::{AudioBackendType, AudioDevice, BackendManager};
use serde_json::Value;
use std::path::PathBuf;

use crate::tui::strings as s;
use crate::tui::theme;

use super::worker_fns::enumerate_devices;

/// The bundle's target backend + a live enumeration for the re-pick picker
/// (mirrors cli/settings.rs `build_live_system`).
pub(super) fn build_live(
    bundle: &qbz_app::settings::bundle::Bundle,
) -> (qbz_app::settings::bundle::LiveSystem, AudioBackendType, Vec<AudioDevice>) {
    let backends: Vec<String> = BackendManager::available_backends()
        .into_iter()
        .filter_map(|b| serde_json::to_value(b).ok().and_then(|v| v.as_str().map(str::to_string)))
        .collect();
    let wanted: Option<AudioBackendType> = bundle
        .domains
        .get("audio")
        .and_then(|a| a.get("backend_type"))
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    let backend = wanted.unwrap_or(AudioBackendType::SystemDefault);
    let devices = enumerate_devices(backend).unwrap_or_default();
    let live_devices: Vec<(String, String)> =
        devices.iter().map(|d| (d.id.clone(), d.name.clone())).collect();
    (
        qbz_app::settings::bundle::LiveSystem { backends, devices: live_devices },
        backend,
        devices,
    )
}

/// Pure footer mapping (tested below). Three states, each spelled out in text —
/// the tone only reinforces it:
/// - unreachable → dim `daemon: not reachable`;
/// - reachable but not signed in → warn `daemon: running · not signed in`
///   (a deliberate FB2 addition over the base footer: an operator-visible
///   needs-auth cue, owner veto at the smoke);
/// - running + signed in → ok, with the optional `playing …` tail.
pub(super) fn footer_state(
    reachable: bool,
    logged_in: bool,
    playing: Option<String>,
) -> (String, ratatui::style::Style) {
    if !reachable {
        (format!(" {}", s::FOOTER_UNREACHABLE), theme::dim())
    } else if !logged_in {
        (
            format!(" {} · {}", s::FOOTER_RUNNING, s::FOOTER_NEEDS_AUTH),
            theme::warn(),
        )
    } else {
        let text = match playing {
            Some(e) => format!(" {} · {e}", s::FOOTER_RUNNING),
            None => format!(" {}", s::FOOTER_RUNNING),
        };
        (text, theme::ok())
    }
}

/// A "playing 192000 Hz / 24 bit" tail from a status body (§4.3), if playing.
pub(super) fn playing_extra(status: &Value) -> Option<String> {
    let state = status.pointer("/playback/state").and_then(Value::as_str).unwrap_or("");
    if state != "playing" {
        return None;
    }
    let sr = status.pointer("/audio/sample_rate").and_then(Value::as_u64)?;
    let bd = status.pointer("/audio/bit_depth").and_then(Value::as_u64)?;
    Some(format!("playing {sr} Hz / {bd} bit"))
}

pub(super) fn desktop_profile_present() -> bool {
    dirs::data_dir().map(|d| d.join("qbz").exists()).unwrap_or(false)
}

pub(super) fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}
