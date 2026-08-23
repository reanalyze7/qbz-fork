//! Building a `SettingsSnapshot` from freshly-read settings, and the
//! blocking top-level `load_snapshot` entry point.

use qbz_audio::backend::{AlsaPlugin, AudioBackendType, BackendManager};

use super::assemble::{assemble_snapshot, SnapshotParts};
use super::types::SettingsSnapshot;
use crate::settings::devices::{enumerate_devices, output_labels};
use crate::settings::store::{with_audio, with_playback, SettingsCtx};
use crate::settings::tables::{ALSA_PLUGINS, RETRY_BEHAVIORS};
use crate::ui_prefs;

/// Read both domain stores, the JSON UI prefs, and enumerate audio
/// devices. Blocking (SQLite + device enumeration) — run inside
/// `spawn_blocking`. Also fills the index maps.
pub fn load_snapshot(ctx: &SettingsCtx) -> SettingsSnapshot {
    let audio = with_audio(&ctx.audio, |s| s.get_settings()).unwrap_or_default();
    let prefs = with_playback(&ctx.playback, |s| s.get_preferences()).unwrap_or_default();
    let ui = ui_prefs::load();
    build_snapshot(ctx, audio, prefs, &ui.streaming_quality)
}

/// Build a snapshot from already-read settings. Splitting this out lets
/// `load_snapshot` and a post-reset rebuild share the device-enumeration
/// and index-mapping logic. The tail (struct-literal assembly) lives in
/// `assemble.rs`.
fn build_snapshot(
    ctx: &SettingsCtx,
    audio: qbz_audio::settings::AudioSettings,
    prefs: qbz_app::settings::playback::PlaybackPreferences,
    streaming_quality_key: &str,
) -> SettingsSnapshot {
    // Keep the session-persistence gates in step with the live playback prefs
    // whenever a settings snapshot is built (startup load + post-reset rebuild).
    crate::session_persist::set_gates(prefs.persist_session, prefs.resume_playback_position);
    let backend_types = BackendManager::available_backends();
    let current_backend = audio.backend_type.unwrap_or_default();
    let backend_index = backend_types
        .iter()
        .position(|t| *t == current_backend)
        .unwrap_or(0);
    let active_backend = backend_types
        .get(backend_index)
        .copied()
        .unwrap_or_default();

    let device_list = enumerate_devices(active_backend);
    let device_index = match &audio.output_device {
        None => 0,
        Some(id) => device_list.ids.iter().position(|d| d == id).unwrap_or(0),
    };

    let alsa_plugin = audio.alsa_plugin.unwrap_or(AlsaPlugin::Hw);
    let alsa_plugin_index = ALSA_PLUGINS
        .iter()
        .position(|(_, p)| *p == alsa_plugin)
        .unwrap_or(0);
    let retry_behavior_index = RETRY_BEHAVIORS
        .iter()
        .position(|(_, v)| *v == audio.quality_fallback_behavior)
        .unwrap_or(0);

    // Detected device limit (#638 fix 3): a cheap cache read — the probe
    // itself only runs on the explicit refresh triggers, never here.
    let (device_cap_summary, device_cap_detected) = crate::device_cap::summary();

    let backend_is_alsa = active_backend == AudioBackendType::Alsa;
    let backend_is_pipewire = active_backend == AudioBackendType::PipeWire;
    let backend_is_jack = active_backend == AudioBackendType::Jack;
    let alsa_plugin_is_hw = alsa_plugin == AlsaPlugin::Hw;
    let (out_backend_label, out_mode_label, out_backend_active, out_mode_active) =
        output_labels(&audio);
    let continue_playback =
        prefs.autoplay_mode == qbz_app::settings::playback::AutoplayMode::ContinueWithinSource;

    {
        let mut maps = ctx.maps.lock().unwrap_or_else(|e| e.into_inner());
        maps.backends = backend_types.clone();
        maps.devices = device_list.ids.clone();
    }

    assemble_snapshot(SnapshotParts {
        audio,
        prefs,
        streaming_quality_key: streaming_quality_key.to_string(),
        backend_types,
        backend_index,
        device_list,
        device_index,
        alsa_plugin_index,
        retry_behavior_index,
        device_cap_summary,
        device_cap_detected,
        backend_is_alsa,
        backend_is_pipewire,
        backend_is_jack,
        alsa_plugin_is_hw,
        continue_playback,
        output_backend_label: out_backend_label,
        output_mode_label: out_mode_label,
        output_backend_active: out_backend_active,
        output_mode_active: out_mode_active,
    })
}
