use qbz_app::settings::daemon_prefs;
use qbz_app::playback_driver;

/// Re-read `audio_settings.db` and apply it to the live `Player`. A struct
/// refresh (`reload_settings`) always happens; the output device is ADDITIONALLY
/// reinitialized only when a routing-critical field actually changed since the
/// last reload (mirrors the desktop's `Apply::Reinit` — `qbz/src/settings.rs:
/// 87-94`, per-key classification `:877-967,1134-1290`; 03-setup-tui.md §4.3
/// lists the same 9 fields).
pub(crate) fn reload_audio(state: &crate::api::ApiState) {
    let fresh = match state.audio.get_settings() {
        Ok(s) => s,
        Err(e) => {
            log::warn!("[reload] could not re-read audio settings: {e}");
            return;
        }
    };
    let player = state.runtime.core().player();
    if let Err(e) = player.reload_settings(fresh.clone()) {
        log::warn!("[reload] player.reload_settings failed: {e}");
    }
    let needs_reinit = state
        .audio_snapshot
        .lock()
        .map(|old| audio_routing_changed(&old, &fresh))
        .unwrap_or(false);
    if needs_reinit {
        log::info!("[reload] routing-critical audio field changed — reinitializing the output device");
        if let Err(e) = player.reinit_device(fresh.output_device.clone()) {
            log::warn!("[reload] player.reinit_device failed: {e}");
        }
    }
    if let Ok(mut snap) = state.audio_snapshot.lock() {
        *snap = fresh;
    }
}

/// The Reinit-class field set (03-setup-tui.md §4.3 / `qbz/src/settings.rs:
/// 877-967,1134-1290`): backend, device, ALSA plugin, DSD mode, max sample
/// rate, exclusive mode, DAC passthrough, hardware volume, lock-output
/// (`skip_sink_switch`). Every other `AudioSettings` field is Reload-class —
/// `player.reload_settings` above already covers it unconditionally.
pub(crate) fn audio_routing_changed(
    old: &qbz_audio::settings::AudioSettings,
    new: &qbz_audio::settings::AudioSettings,
) -> bool {
    old.backend_type != new.backend_type
        || old.output_device != new.output_device
        || old.alsa_plugin != new.alsa_plugin
        || old.alsa_hardware_volume != new.alsa_hardware_volume
        || old.exclusive_mode != new.exclusive_mode
        || old.dac_passthrough != new.dac_passthrough
        || old.skip_sink_switch != new.skip_sink_switch
        || old.dsd_mode != new.dsd_mode
        || old.device_max_sample_rate != new.device_max_sample_rate
}

/// Re-read `daemon_prefs.streaming_quality` into the live cell the driver's
/// background auto-advance reads (`daemon.rs::run`'s `quality_cell`). Manual
/// play/next/prev already re-read `daemon_prefs` fresh every call
/// (`api/playback.rs::resolve_quality`); this is what makes the passive
/// natural-end-of-track advance equally live.
pub(crate) fn reload_quality(state: &crate::api::ApiState) {
    let prefs = daemon_prefs::load_at(&state.roots.data);
    let fresh = playback_driver::quality_from_key(&prefs.streaming_quality);
    if let Ok(mut q) = state.quality.lock() {
        *q = fresh;
    }
}
