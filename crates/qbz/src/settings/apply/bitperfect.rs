//! Forcing local player volume to 100% under bit-perfect ALSA-direct-hw.
use slint::ComponentHandle;

use qbz_app::shell::AppRuntime;
use qbz_audio::backend::{AlsaPlugin, AudioBackendType};

use crate::adapter::SlintAdapter;
use crate::settings::store::{with_audio, SettingsCtx};
use crate::{AppWindow, NowPlayingState};

/// Force the local player volume to 100% in bit-perfect, mirroring Tauri's
/// `playerSetVolume(100)`. Bit-perfect (ALSA backend + `hw` plugin) requires the
/// software volume out of the path, so the player runs at unity gain and the
/// hardware/DAC controls level. Gated on NOT controlling a peer — while a peer
/// renderer owns playback the local lock is lifted and the user adjusts the
/// remote renderer, so forcing local 100 there would be wrong.
///
/// `core().set_volume` is the safe seam (it does NOT touch the protected
/// device-init). Pushes `NowPlayingState.volume = 1.0` so the bar reflects it.
pub(in crate::settings) async fn maybe_force_bitperfect_volume(
    ctx: &SettingsCtx,
    runtime: &AppRuntime<SlintAdapter>,
    weak: &slint::Weak<AppWindow>,
) {
    let audio = match with_audio(&ctx.audio, |s| s.get_settings()) {
        Ok(s) => s,
        Err(e) => {
            log::error!("[qbz-slint] re-read audio for force-100 failed: {e}");
            return;
        }
    };
    let is_alsa_direct_hw = audio.backend_type.unwrap_or_default() == AudioBackendType::Alsa
        && audio.alsa_plugin.unwrap_or(AlsaPlugin::Hw) == AlsaPlugin::Hw;
    if !is_alsa_direct_hw {
        return;
    }
    if let Err(e) = runtime.core().set_volume(1.0) {
        log::error!("[qbz-slint] force bit-perfect volume to 100 failed: {e}");
        return;
    }
    log::info!("[qbz-slint] bit-perfect: forced local volume to 100%");
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<NowPlayingState>().set_volume(1.0);
    });
}

/// Public entry for the startup audio-settings load: apply the bit-perfect
/// force-100 once the player is seeded, so the bar reflects unity gain before
/// the user ever opens Settings. No-op unless ALSA-direct-hw and not controlling.
pub async fn apply_startup_bitperfect_volume(
    ctx: &SettingsCtx,
    runtime: &AppRuntime<SlintAdapter>,
    weak: &slint::Weak<AppWindow>,
) {
    maybe_force_bitperfect_volume(ctx, runtime, weak).await;
}
