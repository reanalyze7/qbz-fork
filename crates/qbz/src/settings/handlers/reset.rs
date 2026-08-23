//! `handle_reset` and `handle_release_device`.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;
use crate::settings::apply::apply_audio;
use crate::settings::snapshot::{apply_snapshot, load_snapshot};
use crate::settings::store::{with_audio, with_playback, Apply, SettingsCtx};
use crate::AppWindow;

/// Reset all Audio + Playback settings to defaults, rebuild the snapshot,
/// push it onto `SettingsState`, and re-apply the audio settings to the
/// player. Streaming Quality (a UI-only pref) is intentionally left
/// untouched — it is not part of either domain store.
pub async fn handle_reset(
    ctx: Arc<SettingsCtx>,
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
) {
    if let Err(e) = with_audio(&ctx.audio, |s| s.reset_all()) {
        log::error!("[qbz-slint] audio reset_all failed: {e}");
    }
    if let Err(e) = with_playback(&ctx.playback, |s| s.reset_all()) {
        log::error!("[qbz-slint] playback reset_all failed: {e}");
    }
    // Reset turns "Limit quality to device" off — drop the cached cap so the
    // next play is uncapped and the snapshot below reads the cleared state
    // (#638 fix 3).
    crate::settings::apply::refresh_device_cap(&ctx, &weak).await;
    // Rebuild the snapshot off the UI thread (device enumeration blocks).
    let snap = {
        let ctx = ctx.clone();
        match tokio::task::spawn_blocking(move || load_snapshot(&ctx)).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("[qbz-slint] settings reset rebuild task failed: {e}");
                return;
            }
        }
    };
    let _ = weak.upgrade_in_event_loop(move |w| {
        apply_snapshot(&w, snap);
    });
    // Routing-critical defaults changed — re-init the device.
    apply_audio(&ctx, &runtime, Apply::Reinit);
}

/// Release the held output device, then re-enumerate. Frees a device QBZ is
/// holding exclusively (ALSA Direct, which leaves the DAC invisible to
/// PipeWire/other apps) and rebuilds the snapshot so a freed or hot-plugged
/// DAC shows up in the list without restarting the app — the Tauri
/// "refresh" affordance plus an explicit release, in one action.
pub async fn handle_release_device(
    ctx: Arc<SettingsCtx>,
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
) {
    if let Err(e) = runtime.core().player().release_device() {
        log::error!("[qbz-slint] player.release_device failed: {e}");
    }
    // Let PipeWire/WirePlumber reclaim the just-freed device before we list.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let snap = {
        let ctx = ctx.clone();
        match tokio::task::spawn_blocking(move || load_snapshot(&ctx)).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("[qbz-slint] release-device rebuild task failed: {e}");
                return;
            }
        }
    };
    let _ = weak.upgrade_in_event_loop(move |w| {
        apply_snapshot(&w, snap);
    });
}
