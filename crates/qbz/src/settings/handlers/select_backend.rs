//! The `handle_select` "backend" arm — split out on its own since it is
//! the largest of the dropdown handlers.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_audio::backend::AudioBackendType;

use crate::adapter::SlintAdapter;
use crate::settings::apply::{
    apply_audio, maybe_force_bitperfect_volume, rebuild_and_push, refresh_device_cap,
};
use crate::settings::store::{with_audio, Apply, SettingsCtx};
use crate::AppWindow;

/// Dropdown index 0 is "Auto" — a resolve-and-set action (#470), not a
/// persisted mode. Pick the best available backend (PipeWire if present,
/// else System), persist it concrete, and let the rebuilt snapshot move
/// the dropdown onto that backend; backend_type is never left null/Auto.
/// Indices >= 1 map to the concrete `maps.backends` list (no Auto entry).
pub(super) async fn select_backend(
    ctx: Arc<SettingsCtx>,
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    index: usize,
) {
    let backend = if index == 0 {
        let types = ctx.maps.lock().unwrap_or_else(|e| e.into_inner()).backends.clone();
        if types.iter().any(|t| *t == AudioBackendType::PipeWire) {
            AudioBackendType::PipeWire
        } else {
            AudioBackendType::SystemDefault
        }
    } else {
        let resolved = ctx
            .maps
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .backends
            .get(index - 1)
            .copied();
        let Some(resolved) = resolved else {
            return;
        };
        resolved
    };
    if let Err(e) = with_audio(&ctx.audio, |s| s.set_backend_type(Some(backend))) {
        log::error!("[qbz-slint] persist backend failed: {e}");
        return;
    }
    // Cross-setting cascades — force settings unsupported by the
    // new backend off, matching the Tauri app.
    if backend != AudioBackendType::PipeWire {
        if let Err(e) = with_audio(&ctx.audio, |s| s.set_dac_passthrough(false)) {
            log::error!("[qbz-slint] cascade dac-passthrough off failed: {e}");
        }
        if let Err(e) = with_audio(&ctx.audio, |s| s.set_pw_force_bitperfect(false)) {
            log::error!("[qbz-slint] cascade pw-force-bitperfect off failed: {e}");
        }
    }
    if backend != AudioBackendType::Alsa {
        if let Err(e) = with_audio(&ctx.audio, |s| s.set_exclusive_mode(false)) {
            log::error!("[qbz-slint] cascade exclusive-mode off failed: {e}");
        }
    }
    if backend == AudioBackendType::Alsa {
        if let Err(e) = with_audio(&ctx.audio, |s| s.set_gapless_enabled(false)) {
            log::error!("[qbz-slint] cascade gapless off failed: {e}");
        }
    }
    // A backend switch invalidates the device list; reset routing
    // to the system default.
    if let Err(e) = with_audio(&ctx.audio, |s| s.set_output_device(None)) {
        log::error!("[qbz-slint] reset output device failed: {e}");
    }
    // Apply to the player first, then rebuild + re-push the full
    // snapshot. `load_snapshot` re-enumerates the new backend's
    // devices and refills the index maps, so the new device list,
    // the reset device index, the forced cascade changes
    // (dac-passthrough / pw-force-bitperfect / exclusive-mode /
    // gapless) and the conditional flags all reach the UI in one
    // consistent push.
    apply_audio(&ctx, &runtime, Apply::Reinit);
    // A backend switch reset the output device to the system default
    // above — re-detect the device cap for it (#638 fix 3) BEFORE the
    // snapshot rebuild below reads the cache.
    refresh_device_cap(&ctx, &weak).await;
    // Bit-perfect (ALSA + hw) forces local volume to 100%; lifted while
    // controlling a peer. Mirrors Tauri's playerSetVolume(100).
    maybe_force_bitperfect_volume(&ctx, &runtime, &weak).await;
    rebuild_and_push(ctx, weak).await;
}
