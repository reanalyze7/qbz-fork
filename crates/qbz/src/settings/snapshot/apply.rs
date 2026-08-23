//! Pushing a `SettingsSnapshot` onto the `SettingsState` Slint global.

use slint::{ModelRc, SharedString, VecModel};

use super::types::SettingsSnapshot;
use crate::{AppWindow, NowPlayingState, SettingsState};

fn string_model(items: Vec<String>) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(
        items
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>(),
    ))
}

fn bool_model(items: Vec<bool>) -> ModelRc<bool> {
    ModelRc::new(VecModel::from(items))
}

/// Push a snapshot onto the `SettingsState` global. Runs on the UI thread.
pub fn apply_snapshot(window: &AppWindow, snap: SettingsSnapshot) {
    let st = window.global::<SettingsState>();
    // Audio — dropdowns.
    st.set_streaming_qualities(string_model(snap.streaming_qualities));
    st.set_streaming_quality_index(snap.streaming_quality_index);
    st.set_backends(string_model(snap.backends));
    st.set_backend_index(snap.backend_index);
    st.set_devices(string_model(snap.devices));
    st.set_device_bp(bool_model(snap.device_bp));
    st.set_device_groups(string_model(snap.device_groups));
    st.set_device_index(snap.device_index);
    st.set_alsa_plugins(string_model(snap.alsa_plugins));
    st.set_alsa_plugin_index(snap.alsa_plugin_index);
    // Audio — toggles.
    st.set_limit_quality_to_device(snap.limit_quality_to_device);
    st.set_device_cap_summary(snap.device_cap_summary.into());
    st.set_device_cap_detected(snap.device_cap_detected);
    st.set_alsa_hardware_volume(snap.alsa_hardware_volume);
    st.set_dsd_modes(string_model(snap.dsd_modes));
    st.set_dsd_mode_index(snap.dsd_mode_index);
    st.set_exclusive_mode(snap.exclusive_mode);
    st.set_reserve_dac(snap.reserve_dac);
    st.set_dac_passthrough(snap.dac_passthrough);
    st.set_pw_force_bitperfect(snap.pw_force_bitperfect);
    st.set_allow_quality_fallback(snap.allow_quality_fallback);
    st.set_sync_audio_on_startup(snap.sync_audio_on_startup);
    st.set_skip_sink_switch(snap.skip_sink_switch);
    // Audio — conditional flags.
    st.set_backend_is_alsa(snap.backend_is_alsa);
    st.set_backend_is_pipewire(snap.backend_is_pipewire);
    st.set_backend_is_jack(snap.backend_is_jack);
    st.set_alsa_plugin_is_hw(snap.alsa_plugin_is_hw);
    // Playback.
    st.set_continue_playback(snap.continue_playback);
    st.set_show_context_icon(snap.show_context_icon);
    st.set_persist_session(snap.persist_session);
    st.set_resume_position(snap.resume_position);
    st.set_gapless(snap.gapless);
    st.set_stream_uncached(snap.stream_uncached);
    st.set_streaming_only(snap.streaming_only);
    st.set_normalization(snap.normalization);
    // Mirror the four output LEDs onto NowPlayingState too, so the Mode C
    // "Small" now-playing bar has a single source for the song card + the
    // DAC/EXC cluster. Cloned because the SettingsState setters below consume
    // the snapshot's String fields via `.into()`.
    let np = window.global::<NowPlayingState>();
    np.set_output_backend_label(snap.output_backend_label.clone().into());
    np.set_output_mode_label(snap.output_mode_label.clone().into());
    np.set_output_backend_active(snap.output_backend_active);
    np.set_output_mode_active(snap.output_mode_active);
    st.set_output_backend_label(snap.output_backend_label.into());
    st.set_output_mode_label(snap.output_mode_label.into());
    st.set_output_backend_active(snap.output_backend_active);
    st.set_output_mode_active(snap.output_mode_active);
    st.set_buffer_seconds(snap.buffer_seconds);
    st.set_crossfade_seconds(snap.crossfade_seconds);
    st.set_retry_behaviors(string_model(snap.retry_behaviors));
    st.set_retry_behavior_index(snap.retry_behavior_index);
    st.set_loading(false);
}
