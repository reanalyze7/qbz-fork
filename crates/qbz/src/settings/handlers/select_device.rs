//! The `handle_select` "device" and "alsa-plugin" arms.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;
use crate::settings::apply::{apply_audio, maybe_force_bitperfect_volume, push_conditional_flags, refresh_device_cap};
use crate::settings::store::{with_audio, Apply, SettingsCtx};
use crate::AppWindow;

pub(super) async fn select_device(
    ctx: Arc<SettingsCtx>,
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    index: usize,
) {
    let id = ctx
        .maps
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .devices
        .get(index)
        .cloned();
    let Some(id) = id else {
        return;
    };
    let device_opt = if id.is_empty() { None } else { Some(id.as_str()) };
    if let Err(e) = with_audio(&ctx.audio, |s| s.set_output_device(device_opt)) {
        log::error!("[qbz-slint] persist device failed: {e}");
        return;
    }
    apply_audio(&ctx, &runtime, Apply::Reinit);
    // The cap is per-device — re-detect for the new output (#638
    // fix 3). No-op while the limit toggle is off.
    refresh_device_cap(&ctx, &weak).await;
}

pub(super) async fn select_alsa_plugin(
    ctx: Arc<SettingsCtx>,
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    plugin: qbz_audio::backend::AlsaPlugin,
) {
    if let Err(e) = with_audio(&ctx.audio, |s| s.set_alsa_plugin(Some(plugin))) {
        log::error!("[qbz-slint] persist ALSA plugin failed: {e}");
        return;
    }
    // ALSA plugin gates the Hardware Volume Control row.
    push_conditional_flags(&ctx, &weak);
    apply_audio(&ctx, &runtime, Apply::Reinit);
    // Switching to/from the `hw` plugin changes bit-perfect status;
    // re-apply the force-100 (no-op when not ALSA-direct-hw).
    maybe_force_bitperfect_volume(&ctx, &runtime, &weak).await;
}
