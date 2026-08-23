//! `DiagController::refresh` (+ its async body) — the settings-read +
//! core-snapshot + row-build + one-event-loop-hop pipeline.

use serde_json::Value;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::DiagnosticsState;

use super::super::collect::gather_blocking;
use super::super::export_json::build_playback_json;
use super::super::rows::{
    build_audio_rows, build_env_rows, build_graphics_rows, build_playback_rows, build_system_rows,
};
use super::super::DiagController;

impl DiagController {
    /// Build the diagnostics snapshot and push the seven models. Called on the UI
    /// thread (Slint callback); flips `loading` immediately, then spawns the work.
    pub(in crate::diagnostics) fn refresh(&self) {
        if let Some(w) = self.weak.upgrade() {
            w.global::<DiagnosticsState>().set_loading(true);
        }
        let this = self.clone();
        self.handle.spawn(async move {
            this.refresh_async().await;
        });
    }

    async fn refresh_async(&self) {
        // (a) blocking: the three settings stores + /proc + /sys reads.
        let collected = tokio::task::spawn_blocking(gather_blocking).await;

        let (runtime_diag, sys, active_output, available_outputs, active_fmt) = match collected {
            Ok(v) => v,
            Err(e) => {
                log::warn!("[qbz-slint] diagnostics: settings read panicked: {e}");
                let weak = self.weak.clone();
                let _ = weak.upgrade_in_event_loop(|w| {
                    let d = w.global::<DiagnosticsState>();
                    d.set_loading(false);
                    d.set_error("Failed to read diagnostics".into());
                });
                return;
            }
        };

        // (b) async core snapshot for the Playback section.
        let pb = self.runtime.core().get_playback_state();
        let track = self.runtime.core().current_track().await;

        // (d) build the row vectors (1:1 with the Tauri row builders).
        let system_rows = build_system_rows(&sys);
        let playback_rows = build_playback_rows(&pb, track.as_ref());
        let audio_rows = build_audio_rows(
            &runtime_diag,
            active_output.as_deref(),
            &available_outputs,
            active_fmt
                .as_ref()
                .map(|(r, _)| r.as_str())
                .filter(|s| !s.is_empty()),
            active_fmt
                .as_ref()
                .map(|(_, f)| f.as_str())
                .filter(|s| !s.is_empty()),
        );
        let graphics_rows = build_graphics_rows(&runtime_diag);
        let env_rows = build_env_rows(&runtime_diag);

        // (e) cache the export base (runtimeDiag flattened + systemInfo +
        //     playback). exportedAt is added at export.
        let playback_json = build_playback_json(&pb, track.as_ref());
        let mut map = serde_json::Map::new();
        if let Ok(Value::Object(rd)) = serde_json::to_value(&runtime_diag) {
            for (k, v) in rd {
                map.insert(k, v);
            }
        }
        map.insert(
            "systemInfo".to_string(),
            serde_json::to_value(&sys).unwrap_or(Value::Null),
        );
        map.insert("playback".to_string(), playback_json);
        if let Ok(mut g) = self.export.lock() {
            *g = Some(Value::Object(map));
        }

        let app_version = runtime_diag.app_version.clone();

        // (f) one event-loop hop: push all models + version + flags.
        let weak = self.weak.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            let d = w.global::<DiagnosticsState>();
            d.set_system_rows(ModelRc::new(VecModel::from(system_rows)));
            d.set_playback_rows(ModelRc::new(VecModel::from(playback_rows)));
            d.set_audio_rows(ModelRc::new(VecModel::from(audio_rows)));
            d.set_graphics_rows(ModelRc::new(VecModel::from(graphics_rows)));
            d.set_env_rows(ModelRc::new(VecModel::from(env_rows)));
            d.set_app_version(app_version.into());
            d.set_loaded(true);
            d.set_loading(false);
            d.set_error("".into());
        });
    }
}
