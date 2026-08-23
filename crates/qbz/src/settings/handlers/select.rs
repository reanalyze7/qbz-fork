//! `handle_select` — dropdown-change dispatch, plus `handle_string`.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;

use super::select_backend::select_backend;
use super::select_device::{select_alsa_plugin, select_device};
use crate::adapter::SlintAdapter;
use crate::settings::apply::apply_audio;
use crate::settings::store::{with_audio, Apply, SettingsCtx};
use crate::settings::tables::{ALSA_PLUGINS, DSD_MODES, RETRY_BEHAVIORS};
use crate::ui_prefs::{self, STREAMING_QUALITIES};
use crate::AppWindow;

/// Handle a dropdown change: persist it, apply audio ones to the player,
/// and — for a backend switch — re-enumerate devices and recompute the
/// conditional flags into `SettingsState`.
pub async fn handle_select(
    ctx: Arc<SettingsCtx>,
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    key: String,
    index: usize,
) {
    match key.as_str() {
        "streaming-quality" => {
            // UI-only preference, persisted to ui_prefs.json.
            let Some(quality) = STREAMING_QUALITIES.get(index) else {
                return;
            };
            let mut prefs = ui_prefs::load();
            if prefs.streaming_quality != quality.key {
                prefs.streaming_quality = quality.key.to_string();
                ui_prefs::save(&prefs);
                // The L1/L2 audio cache is keyed by track id alone (no quality
                // dimension), so bytes fetched at the old tier would keep
                // serving plays and casts until they aged out. Clear it so the
                // new tier applies from the next fetch — fire-and-forget even
                // mid-playback: an armed gapless handoff may drop (one possible
                // audible gap on this explicit, rare action), which beats
                // intermittently serving the old tier. Logged so a reported
                // gap is attributable.
                log::info!(
                    "[qbz-slint] streaming quality changed -> clearing audio cache (L1+L2)"
                );
                runtime.core().player().clear_audio_cache();
            }
        }
        "backend" => select_backend(ctx, runtime, weak, index).await,
        "device" => select_device(ctx, runtime, weak, index).await,
        "dsd-mode" => {
            let Some((_, mode)) = DSD_MODES.get(index) else {
                return;
            };
            if let Err(e) = with_audio(&ctx.audio, |s| s.set_dsd_mode(mode)) {
                log::error!("[qbz-slint] persist DSD mode failed: {e}");
                return;
            }
            apply_audio(&ctx, &runtime, Apply::Reinit);
        }
        "alsa-plugin" => {
            let plugin = ALSA_PLUGINS.get(index).map(|(_, p)| *p);
            let Some(plugin) = plugin else {
                return;
            };
            select_alsa_plugin(ctx, runtime, weak, plugin).await;
        }
        "retry-behavior" => {
            let behavior = RETRY_BEHAVIORS.get(index).map(|(_, v)| *v).unwrap_or("ask");
            if let Err(e) = with_audio(&ctx.audio, |s| s.set_quality_fallback_behavior(behavior)) {
                log::error!("[qbz-slint] persist retry behavior failed: {e}");
                return;
            }
            apply_audio(&ctx, &runtime, Apply::Reload);
        }
        other => log::warn!("[qbz-slint] unknown settings select key: {other}"),
    }
}

/// Handle a text-input commit (Enter or focus loss). No text setting is wired
/// right now — the seam is kept so a future one plugs in without re-plumbing
/// the Slint callback chain.
pub async fn handle_string(_weak: slint::Weak<AppWindow>, key: String, _value: String) {
    log::warn!("[qbz-slint] unknown settings string key: {key}");
}
