//! `handle_bool` — the toggle-callback entry point: cross-setting
//! cascades, then per-key persistence, then live-apply + UI mirroring.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;

use super::bool_keys::persist_bool_key;
use super::offline::set_offline_mode;
use crate::adapter::SlintAdapter;
use crate::settings::apply::{apply_audio, rebuild_and_push};
use crate::settings::store::{with_audio, Apply, SettingsCtx};
use crate::{AppWindow, SettingsState};

/// Handle a toggle change: persist it, apply any cross-setting cascade,
/// then apply audio settings to the player.
///
/// Cross-setting cascades (matching the Tauri app):
///  - DAC passthrough ON  → force `skip_sink_switch` off (mutually exclusive).
///  - DAC passthrough OFF → force `pw_force_bitperfect` off.
///  - Streaming-only  ON  → force `gapless_enabled` off.
///
/// When a cascade fires, the forced changes are persisted too and the
/// whole snapshot is rebuilt and re-pushed to `SettingsState` so the UI
/// (toggle states, conditional rows, disabled states) stays consistent.
pub async fn handle_bool(
    ctx: Arc<SettingsCtx>,
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    key: String,
    value: bool,
) {
    let key = key.as_str();
    // The offline-MODE toggle persists through the shared engine's per-user
    // store, not the audio/playback stores — routed apart from the Apply
    // machinery below.
    if key == "offline-mode-enabled" {
        set_offline_mode(ctx, runtime, weak, value).await;
        return;
    }
    // Cross-setting cascades — force dependent settings off and persist
    // those forced changes. `cascaded` flags whether a full snapshot
    // re-push is needed afterwards.
    let mut cascaded = false;
    match key {
        "dac-passthrough" if value => {
            if let Err(e) = with_audio(&ctx.audio, |s| s.set_skip_sink_switch(false)) {
                log::error!("[qbz-slint] cascade skip-sink-switch off failed: {e}");
            } else {
                cascaded = true;
            }
        }
        "dac-passthrough" => {
            if let Err(e) = with_audio(&ctx.audio, |s| s.set_pw_force_bitperfect(false)) {
                log::error!("[qbz-slint] cascade pw-force-bitperfect off failed: {e}");
            } else {
                cascaded = true;
            }
        }
        "streaming-only" if value => {
            if let Err(e) = with_audio(&ctx.audio, |s| s.set_gapless_enabled(false)) {
                log::error!("[qbz-slint] cascade gapless off failed: {e}");
            } else {
                cascaded = true;
            }
        }
        _ => {}
    }

    let outcome = persist_bool_key(&ctx, &runtime, key, value).await;
    match outcome {
        Ok(apply) => {
            // A cascade forced extra changes — always re-init the device
            // (cascade targets are routing-critical) regardless of what the
            // triggering toggle alone required.
            let apply = if cascaded { Apply::Reinit } else { apply };
            apply_audio(&ctx, &runtime, apply);
            // Reflect the persisted value back onto SettingsState so toggles
            // that are purely driven by `checked: SettingsState.x` (e.g. the
            // now-playing bar audio-menu QbzToggles for Normalization/Gapless)
            // actually flip. The Settings panel's own toggles already reflect
            // their click optimistically, but the bar flyout's do NOT self-flip
            // — they need this push. Skipped when cascaded, since the full
            // snapshot re-push below already carries the new value.
            if !cascaded {
                let key = key.to_string();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let st = w.global::<SettingsState>();
                    match key.as_str() {
                        "normalization" => st.set_normalization(value),
                        "gapless" => st.set_gapless(value),
                        _ => {}
                    }
                });
            }
        }
        Err(e) => log::error!("[qbz-slint] failed to persist '{key}': {e}"),
    }
    // The local device-cap cache follows the toggle (#638 fix 3): re-probe on
    // enable, drop on disable, then re-push the detected-limit row. After the
    // persist above so the refresh reads the fresh flag.
    if key == "limit-quality-to-device" {
        crate::settings::apply::refresh_device_cap(&ctx, &weak).await;
    }
    // After a cascade, rebuild + re-push the full snapshot so the forced
    // changes and disabled states reach the UI.
    if cascaded {
        rebuild_and_push(ctx, weak).await;
    }
}
