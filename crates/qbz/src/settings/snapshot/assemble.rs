//! Assembling the final `SettingsSnapshot` struct literal from the values
//! `build.rs` computed. Split out purely to keep each file under the
//! line-count budget — this is the tail half of `build_snapshot`.

use qbz_audio::settings::AudioSettings;

use super::types::SettingsSnapshot;
use crate::settings::devices::backend_label;
use crate::settings::tables::{ALSA_PLUGINS, DSD_MODES, RETRY_BEHAVIORS};
use crate::ui_prefs::{self, STREAMING_QUALITIES};

/// Everything `assemble_snapshot` needs, computed by `build_snapshot`.
pub(super) struct SnapshotParts {
    pub(super) audio: AudioSettings,
    pub(super) prefs: qbz_app::settings::playback::PlaybackPreferences,
    pub(super) streaming_quality_key: String,
    pub(super) backend_types: Vec<qbz_audio::backend::AudioBackendType>,
    pub(super) backend_index: usize,
    pub(super) device_list: crate::settings::devices::DeviceList,
    pub(super) device_index: usize,
    pub(super) alsa_plugin_index: usize,
    pub(super) retry_behavior_index: usize,
    pub(super) device_cap_summary: String,
    pub(super) device_cap_detected: bool,
    pub(super) backend_is_alsa: bool,
    pub(super) backend_is_pipewire: bool,
    pub(super) backend_is_jack: bool,
    pub(super) alsa_plugin_is_hw: bool,
    pub(super) continue_playback: bool,
    pub(super) output_backend_label: String,
    pub(super) output_mode_label: String,
    pub(super) output_backend_active: bool,
    pub(super) output_mode_active: bool,
}

pub(super) fn assemble_snapshot(p: SnapshotParts) -> SettingsSnapshot {
    let audio = p.audio;
    let prefs = p.prefs;
    SettingsSnapshot {
        streaming_qualities: STREAMING_QUALITIES
            .iter()
            .map(|q| q.label.to_string())
            .collect(),
        streaming_quality_index: ui_prefs::streaming_quality_index(&p.streaming_quality_key) as i32,
        // Index 0 is "Auto" (a resolve-and-set action, #470); the concrete
        // backends follow. backend_type is always persisted concrete, so the
        // current selection is its position shifted by 1 past the Auto entry —
        // the dropdown never rests on Auto.
        backends: std::iter::once(qbz_i18n::t("Auto"))
            .chain(p.backend_types.iter().map(|t| backend_label(*t)))
            .collect(),
        backend_index: p.backend_index as i32 + 1,
        devices: p.device_list.labels,
        device_bp: p.device_list.bp,
        device_groups: p.device_list.groups,
        device_index: p.device_index as i32,
        alsa_plugins: ALSA_PLUGINS.iter().map(|(l, _)| qbz_i18n::t(l)).collect(),
        alsa_plugin_index: p.alsa_plugin_index as i32,
        limit_quality_to_device: audio.limit_quality_to_device,
        device_cap_summary: p.device_cap_summary,
        device_cap_detected: p.device_cap_detected,
        alsa_hardware_volume: audio.alsa_hardware_volume,
        dsd_modes: DSD_MODES.iter().map(|(l, _)| qbz_i18n::t(l)).collect(),
        dsd_mode_index: DSD_MODES
            .iter()
            .position(|(_, v)| *v == audio.dsd_mode)
            .unwrap_or(0) as i32,
        exclusive_mode: audio.exclusive_mode,
        reserve_dac: audio.reserve_dac_while_running,
        dac_passthrough: audio.dac_passthrough,
        pw_force_bitperfect: audio.pw_force_bitperfect,
        allow_quality_fallback: audio.allow_quality_fallback,
        sync_audio_on_startup: audio.sync_audio_on_startup,
        skip_sink_switch: audio.skip_sink_switch,
        backend_is_alsa: p.backend_is_alsa,
        backend_is_pipewire: p.backend_is_pipewire,
        backend_is_jack: p.backend_is_jack,
        alsa_plugin_is_hw: p.alsa_plugin_is_hw,
        continue_playback: p.continue_playback,
        show_context_icon: prefs.show_context_icon,
        persist_session: prefs.persist_session,
        resume_position: prefs.resume_playback_position,
        gapless: audio.gapless_enabled,
        stream_uncached: audio.stream_first_track,
        streaming_only: audio.streaming_only,
        normalization: audio.normalization_enabled,
        buffer_seconds: audio.stream_buffer_seconds as i32,
        crossfade_seconds: audio.crossfade_seconds.round() as i32,
        retry_behaviors: RETRY_BEHAVIORS.iter().map(|(l, _)| qbz_i18n::t(l)).collect(),
        retry_behavior_index: p.retry_behavior_index as i32,
        output_backend_label: p.output_backend_label,
        output_mode_label: p.output_mode_label,
        output_backend_active: p.output_backend_active,
        output_mode_active: p.output_mode_active,
    }
}
