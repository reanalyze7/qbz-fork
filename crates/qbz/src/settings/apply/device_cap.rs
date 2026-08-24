//! Re-detecting the local output device's quality cap and pushing the
//! conditional backend/ALSA flags.
use slint::ComponentHandle;

use crate::settings::store::{with_audio, SettingsCtx};
use crate::settings::tables::ALSA_PLUGINS;
use crate::{AppWindow, SettingsState};
use qbz_audio::backend::{AlsaPlugin, AudioBackendType};

/// Re-detect the local output device's quality cap (#638 fix 3) from the
/// persisted audio settings and re-push the Settings "Detected device limit"
/// row. The probe itself runs off-thread inside `device_cap::refresh`;
/// await-able so callers sequence the UI push after the cache settles.
/// Explicit triggers ONLY — startup, the limit toggle, an output-device or
/// backend change, reset-to-defaults — never the playback path or poll tick.
pub async fn refresh_device_cap(ctx: &SettingsCtx, weak: &slint::Weak<AppWindow>) {
    let audio = match with_audio(&ctx.audio, |s| s.get_settings()) {
        Ok(s) => s,
        Err(e) => {
            log::error!("[qbz-slint] re-read audio settings for device cap failed: {e}");
            return;
        }
    };
    crate::device_cap::refresh(audio.limit_quality_to_device, audio.output_device).await;
    let (summary, detected) = crate::device_cap::summary();
    let _ = weak.upgrade_in_event_loop(move |w| {
        let st = w.global::<SettingsState>();
        st.set_device_cap_summary(summary.into());
        st.set_device_cap_detected(detected);
    });
}

/// Recompute the backend/ALSA conditional flags from the current audio
/// settings and push them onto `SettingsState`. Called after a backend or
/// ALSA-plugin change so the `.slint` panels re-gate the conditional rows.
pub(in crate::settings) fn push_conditional_flags(ctx: &SettingsCtx, weak: &slint::Weak<AppWindow>) {
    let audio = match with_audio(&ctx.audio, |s| s.get_settings()) {
        Ok(s) => s,
        Err(e) => {
            log::error!("[qbz-slint] re-read audio settings for flags failed: {e}");
            return;
        }
    };
    let backend = audio.backend_type.unwrap_or_default();
    let plugin = audio.alsa_plugin.unwrap_or(AlsaPlugin::Hw);
    let is_alsa = backend == AudioBackendType::Alsa;
    let is_pipewire = backend == AudioBackendType::PipeWire;
    let is_jack = backend == AudioBackendType::Jack;
    let plugin_is_hw = plugin == AlsaPlugin::Hw;
    let plugin_index = ALSA_PLUGINS
        .iter()
        .position(|(_, p)| *p == plugin)
        .unwrap_or(0) as i32;
    let _ = weak.upgrade_in_event_loop(move |w| {
        let st = w.global::<SettingsState>();
        st.set_backend_is_alsa(is_alsa);
        st.set_backend_is_pipewire(is_pipewire);
        st.set_backend_is_jack(is_jack);
        st.set_alsa_plugin_is_hw(plugin_is_hw);
        st.set_alsa_plugin_index(plugin_index);
    });
}
