//! Re-reading persisted audio settings and applying them to the live
//! `Player`.

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;
use crate::settings::store::{with_audio, Apply, SettingsCtx};

/// Re-read the persisted audio settings and apply them to the live player.
pub(in crate::settings) fn apply_audio(
    ctx: &SettingsCtx,
    runtime: &AppRuntime<SlintAdapter>,
    apply: Apply,
) {
    let reinit = match apply {
        Apply::None => return,
        Apply::Reload => false,
        Apply::Reinit => true,
    };
    let fresh = match with_audio(&ctx.audio, |s| s.get_settings()) {
        Ok(s) => s,
        Err(e) => {
            log::error!("[qbz-slint] re-read audio settings failed: {e}");
            return;
        }
    };
    let player = runtime.core().player();
    if let Err(e) = player.reload_settings(fresh.clone()) {
        log::error!("[qbz-slint] player.reload_settings failed: {e}");
    }
    if reinit {
        if let Err(e) = player.reinit_device(fresh.output_device.clone()) {
            log::error!("[qbz-slint] player.reinit_device failed: {e}");
        }
    }
    log::info!("[qbz-slint] audio settings applied to player (reinit={reinit})");
}
