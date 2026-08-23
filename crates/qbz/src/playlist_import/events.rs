//! Progress-sink event handling: `apply_event` + the `SlintSink` bridge.

use slint::ComponentHandle;

use qbz_playlist_import::{ImportEvent, ImportPhase, ImportProgressSink};

use crate::{AppWindow, PlaylistImportState};

use super::format::{group_thousands, push_log};
use super::session::{current_generation, SESSION};

/// One sink event onto the modal — the two Svelte event listeners. The
/// generation is checked by the caller (SlintSink). Event-loop thread.
pub fn apply_event(window: &AppWindow, event: ImportEvent) {
    let state = window.global::<PlaylistImportState>();
    match event {
        ImportEvent::Phase(phase) => match phase {
            ImportPhase::Matching => {
                push_log(window, qbz_i18n::t("Searching Qobuz catalog..."), "info");
            }
            // Creating / Adding re-fire once per created part — log each,
            // as Tauri does.
            ImportPhase::Creating => {
                push_log(window, qbz_i18n::t("Creating playlist..."), "success");
            }
            ImportPhase::Adding => {
                push_log(window, qbz_i18n::t("Adding tracks to playlist..."), "info");
            }
        },
        ImportEvent::Progress(p) => {
            // Bar + status update on EVERY event (Tauri parity, no
            // coalescing).
            state.set_has_progress(p.total > 0);
            if p.total > 0 {
                state.set_progress(p.current as f32 / p.total as f32);
            }
            if p.phase == "adding" {
                // Status line per phase — deliberate owner deviation from
                // the Tauri modal, which reused the "Matching tracks…"
                // string here (see qbz_playlist_import::sink::ImportPhase).
                let line = qbz_i18n::t_args("Adding tracks: {} / {}", &[&p.current.to_string(), &p.total.to_string()]);
                state.set_status_line(line.as_str().into());
                // One log line per 50-track chunk event (chunk counts,
                // not tracks) — Tauri logs every adding event.
                push_log(window, line, "info");
            } else if p.total > 0 {
                let line = qbz_i18n::t_args(
                    "Matching tracks: {} / {} ({} found)",
                    &[
                        &group_thousands(p.current),
                        &group_thousands(p.total),
                        &group_thousands(p.matched_so_far),
                    ],
                );
                state.set_status_line(line.as_str().into());
                // Matching is high-frequency (one event per track): log
                // only at 5% milestones, exactly like the Svelte listener.
                let pct = (p.current as u64 * 100 / p.total as u64) as i32;
                let should_log = {
                    let mut s = SESSION.lock().unwrap();
                    if pct >= s.last_logged_percent + 5 {
                        s.last_logged_percent = pct;
                        true
                    } else {
                        false
                    }
                };
                if should_log {
                    push_log(window, line, "info");
                }
            }
            state.set_current_track(p.current_track.unwrap_or_default().into());
        }
    }
}

/// Streams crate events onto the modal via the established
/// `upgrade_in_event_loop` cross-thread hop — one hop per event, the same
/// frequency profile as the artwork/scan pipelines (Tauri also updated
/// the bar per event, no coalescing).
pub struct SlintSink {
    weak: slint::Weak<AppWindow>,
    generation: u64,
}

impl SlintSink {
    pub fn new(weak: slint::Weak<AppWindow>, generation: u64) -> Self {
        Self { weak, generation }
    }
}

impl ImportProgressSink for SlintSink {
    fn emit(&self, event: ImportEvent) {
        let generation = self.generation;
        let _ = self.weak.upgrade_in_event_loop(move |w| {
            // Stale generation = the modal was reset (reopened) while
            // this run was in flight — its events must never touch the
            // fresh modal state (§1.8).
            if generation == current_generation() {
                apply_event(&w, event);
            }
        });
    }
}
