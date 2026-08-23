//! Settings > Offline — the Enable Offline Mode toggle (induced offline).

use std::sync::Arc;

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;
use crate::settings::apply::{apply_audio, rebuild_and_push};
use crate::settings::store::{with_audio, Apply, SettingsCtx};
use crate::{AppWindow, SettingsState};

/// The Enable Offline Mode toggle (induced offline).
///
/// The shared engine persists the flag, handles the #279 stream-first
/// snapshot/restore against the real audio store, and broadcasts the
/// status change (the offline_mode UI forwarder updates `OfflineState`).
/// Exit is ALWAYS allowed — no confirm dialog, no network-probe gate
/// (deliberate improvement over Tauri's exit trap, spec §3.1).
pub(super) async fn set_offline_mode(
    ctx: Arc<SettingsCtx>,
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    value: bool,
) {
    let engine = crate::offline_mode::engine();
    // Best-effort #279: pass the real audio store when it's open (the
    // outer Ok-wrap keeps store-unavailability distinct from engine
    // failures); flip the mode without the snapshot when it isn't.
    let result = match with_audio(&ctx.audio, |store| Ok(engine.set_induced(value, Some(store)))) {
        Ok(inner) => inner,
        Err(e) => {
            log::warn!(
                "[qbz-slint] offline toggle: audio store unavailable for the #279 snapshot ({e}); flipping without it"
            );
            engine.set_induced(value, None)
        }
    };
    match result {
        Ok(status) => {
            log::info!(
                "[qbz-slint] offline mode toggled: induced={value} (mode={:?})",
                status.mode
            );
            // The #279 snapshot may have mutated stream_first_track behind
            // the Playback panel's back: reload the live player settings and
            // re-push the full snapshot so the "Stream uncached tracks"
            // toggle stays honest (Tauri's audio-settings-changed re-sync).
            apply_audio(&ctx, &runtime, Apply::Reload);
            rebuild_and_push(ctx, weak).await;
        }
        Err(e) => {
            log::error!("[qbz-slint] offline mode toggle failed: {e}");
            // Revert the optimistic toggle to the engine's actual state.
            let actual = engine
                .settings()
                .map(|s| s.manual_offline_mode)
                .unwrap_or_else(|_| {
                    engine.status().mode == qbz_app::offline_mode::OfflineMode::InducedOffline
                });
            let _ = weak.upgrade_in_event_loop(move |w| {
                w.global::<SettingsState>().set_offline_mode_enabled(actual);
            });
        }
    }
}
