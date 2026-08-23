// crates/qbzd/src/cli/settings/store.rs — opening the backing stores and
// reading every canonical key's current value (`settings show`'s backing).

use qbz_app::settings::daemon_prefs;
use qbz_app::settings::playback::PlaybackPreferencesStore;
use qbz_audio::settings::AudioSettingsStore;

use crate::paths::ProfileRoots;

use super::codec_bool::{render_alsa_plugin, render_backend, render_bool};
use super::codec_playback::render_autoplay;
use super::codec_value::{render_opt_string, render_opt_u32};
use super::keys::KEY_TABLE;

pub(super) fn open_audio(roots: &ProfileRoots) -> Result<AudioSettingsStore, String> {
    AudioSettingsStore::new_at(&roots.data)
}
pub(super) fn open_playback(roots: &ProfileRoots) -> Result<PlaybackPreferencesStore, String> {
    PlaybackPreferencesStore::new_at(&roots.data)
}

/// Read every canonical key's current value, in [`KEY_TABLE`] order — the
/// backing for `settings show`. Opens each store once (not once per key).
pub(super) fn read_all(roots: &ProfileRoots) -> Result<Vec<(&'static str, String)>, String> {
    let audio = open_audio(roots)?.get_settings()?;
    let playback = open_playback(roots)?.get_preferences()?;
    let prefs = daemon_prefs::load_at(&roots.data);

    let mut out = Vec::with_capacity(KEY_TABLE.len());
    for (key, _) in KEY_TABLE {
        let value = match *key {
            "audio.backend" => render_backend(audio.backend_type),
            "audio.device" => render_opt_string(&audio.output_device),
            "audio.alsa_plugin" => render_alsa_plugin(audio.alsa_plugin),
            "audio.alsa_hardware_volume" => render_bool(audio.alsa_hardware_volume),
            "audio.exclusive_mode" => render_bool(audio.exclusive_mode),
            "audio.dac_passthrough" => render_bool(audio.dac_passthrough),
            "audio.skip_sink_switch" => render_bool(audio.skip_sink_switch),
            "audio.dsd_mode" => audio.dsd_mode.clone(),
            "audio.device_max_sample_rate" => render_opt_u32(audio.device_max_sample_rate),
            "audio.stream_first_track" => render_bool(audio.stream_first_track),
            "audio.stream_buffer_seconds" => audio.stream_buffer_seconds.to_string(),
            "audio.streaming_only" => render_bool(audio.streaming_only),
            "audio.limit_quality_to_device" => render_bool(audio.limit_quality_to_device),
            "audio.allow_quality_fallback" => render_bool(audio.allow_quality_fallback),
            "audio.quality_fallback_behavior" => audio.quality_fallback_behavior.clone(),
            "audio.gapless_enabled" => render_bool(audio.gapless_enabled),
            "audio.normalization_enabled" => render_bool(audio.normalization_enabled),
            "audio.normalization_target_lufs" => audio.normalization_target_lufs.to_string(),
            "audio.pw_force_bitperfect" => render_bool(audio.pw_force_bitperfect),
            "audio.reserve_dac_while_running" => render_bool(audio.reserve_dac_while_running),
            "audio.sync_audio_on_startup" => render_bool(audio.sync_audio_on_startup),
            "playback.quality" => prefs.streaming_quality.clone(),
            "playback.autoplay" => render_autoplay(playback.autoplay_mode),
            "playback.persist_session" => render_bool(playback.persist_session),
            "playback.resume_playback_position" => render_bool(playback.resume_playback_position),
            "playback.show_context_icon" => render_bool(playback.show_context_icon),
            "playback.mpris" => render_bool(prefs.mpris_enabled),
            other => unreachable!("KEY_TABLE/read_all drifted apart on key: {other}"),
        };
        out.push((*key, value));
    }
    Ok(out)
}
