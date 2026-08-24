use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch25(
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
                ("playlist", "unfollow") => {
                    // Card-only: unfollow (unsubscribe) the given playlist.
                    if let Some(w) = weak.upgrade() {
                        if local_playlist::is_local_id(&id) {
                            return;
                        }
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let is_open = w.global::<PlaylistState>().get_id().to_string() == id;
                        playlist_set_follow_by_id(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            pid,
                            false,
                            is_open,
                        );
                    }
                }
                ("playlist", "select-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let on = w.global::<PlaylistState>().get_multi_select_mode();
                        playlist::set_multi_select(&w, !on);
                    }
                }
                ("playlist", "select-all") => {
                    if let Some(w) = weak.upgrade() {
                        playlist::select_all(&w);
                    }
                }
                ("playlist", "play-next-selected") | ("playlist", "queue-selected") => {
                    // Bulk Play next / Add to queue over the selection
                    // (Tauri's BulkActionBar split-button, spec §1.5) —
                    // source-aware: rows resolve through the merged queue
                    // snapshot (local/cached keep their source — the
                    // T2 fix-forward) or the pure-Qobuz Track cache.
                    if let Some(w) = weak.upgrade() {
                        let next = action == "play-next-selected";
                        let tracks = playlist::selected_queue_tracks(&w);
                        if tracks.is_empty() {
                            toast::error(&w, "Nothing playable in the selection");
                            return;
                        }
                        // Selection clears, mode stays on (LL precedent).
                        playlist::clear_selection(&w);
                        playback::enqueue_queue_tracks(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            next,
                        );
                    }
                }
                ("playlist", "add-selected-to-playlist") => {
                    // Bulk Add to playlist (spec §1.5). The picker is
                    // single-mode (catalog ids XOR local-mode refs), so:
                    // Qobuz rows ride the catalog flow; a selection with NO
                    // Qobuz rows rides the local-mode flow (library row ids
                    // — per-row parity for sidecar rows); a MIXED selection
                    // follows Tauri (Qobuz rows only, sidecar rows skipped +
                    // logged).
                    let Some(w) = weak.upgrade() else { return };
                    let rows = playlist::selected_rows(&w);
                    if rows.is_empty() {
                        return;
                    }
                    let mut qobuz_ids: Vec<String> = Vec::new();
                    let mut local_refs: Vec<String> = Vec::new();
                    for row in &rows {
                        match row.source.as_str() {
                            "local" => local_refs.push(row.id.clone()),
                            _ => {
                                if row.id.parse::<u64>().is_ok() {
                                    qobuz_ids.push(row.id.clone());
                                }
                            }
                        }
                    }
                    if !qobuz_ids.is_empty() {
                        if !local_refs.is_empty() {
                            log::info!(
                                "[qbz-slint] bulk add-to-playlist: mixed selection — {} sidecar row(s) skipped (single-mode picker; Tauri §1.5 behavior)",
                                local_refs.len()
                            );
                        }
                        playlist_picker::open_multi(&w, &qobuz_ids, false);
                    } else if !local_refs.is_empty() {
                        playlist_picker::open_multi(&w, &local_refs, true);
                    } else {
                        return;
                    }
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    handle.spawn(async move {
                        let playlists = playlist_picker::load(&runtime).await;
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            playlist_picker::apply(&w, playlists);
                        });
                    });
                }
        _ => {}
    }
}
