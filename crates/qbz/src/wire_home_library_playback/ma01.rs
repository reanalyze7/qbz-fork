use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch01(
    kind: &str,
    id: &str,
    action: &str,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) {
    let runtime = runtime.clone();
    let weak = weak.clone();
    let handle = handle.clone();
    let image_cache = image_cache.clone();
    let id = id.to_string();
    match (kind, action) {
                ("npb-large", "viz-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let shell = w.global::<ShellState>();
                        let on = !shell.get_large_visualizer_on();
                        shell.set_large_visualizer_on(on);
                        let mut prefs = crate::ui_prefs::load();
                        prefs.large_visualizer = on;
                        crate::ui_prefs::save(&prefs);
                    }
                }
                // Large dock: cycle the spectrum visualization (Bars -> Waveform
                // -> Energy), persisted in ui_prefs.
                ("npb-large", "spectrum-cycle") => {
                    if let Some(w) = weak.upgrade() {
                        let shell = w.global::<ShellState>();
                        let next = (shell.get_large_spectrum_mode() + 1).rem_euclid(3);
                        shell.set_large_spectrum_mode(next);
                        let mut prefs = crate::ui_prefs::load();
                        prefs.large_spectrum_mode =
                            crate::ui_prefs::large_spectrum_mode_key(next).to_string();
                        crate::ui_prefs::save(&prefs);
                    }
                }
                // Track Info modal — opened from the NPB (i) button, the
                // song-card title, or a TrackRow context menu. Qobuz tracks
                // only (the id must be a real catalog u64).
                ("track", "track-info") => {
                    if let Ok(track_id) = id.parse::<u64>() {
                        info_modals::open_track_info(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                // "Reveal in file explorer" — local tracks only (the row's
                // id is a library row id, not a catalog id; TrackContextMenu
                // gates the menu entry itself on source == "local").
                // Try the in-memory Tracks-tab cache first (no DB hit);
                // folder-detail rows that aren't in it fall back to an
                // off-thread DB resolve, mirroring the play-next/queue arm
                // just above this match's local block.
                ("track", "reveal-in-explorer") => {
                    if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                        reveal_in_file_manager(&row.file_path);
                    } else if let Ok(rid) = id.parse::<i64>() {
                        handle.spawn(async move {
                            let row = tokio::task::spawn_blocking(move || {
                                crate::library_db::with_db(|db| db.get_track(rid)).flatten()
                            })
                            .await
                            .ok()
                            .flatten();
                            if let Some(row) = row {
                                reveal_in_file_manager(&row.file_path);
                            }
                        });
                    }
                }
                // Album Info (Credits/Review) modal — opened from the album
                // header (i) button. Qobuz albums only (skip local keys).
        _ => {}
    }
}
