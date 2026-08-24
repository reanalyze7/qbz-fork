use crate::*;

/// "Go to album" / "Go to artist" for a LOCAL-surface track row
/// (LocalLibrary Tracks tab / folder detail / local album detail) — an
/// owner improvement over Tauri, which omits both entries on local rows.
/// Source-routed (same split as the MyQBZ artist links and the real-id
/// favorite entry):
///   - local rows -> the LOCAL album view by the row's `album_group_key`
///     (the same navigation key the now-playing bar's "Go to album" uses)
///     / the LocalLibrary Artists tab by NAME (local artists have no id).
///   - qobuz_download rows -> the REAL Qobuz pages. The library index
///     carries ONLY `qobuz_track_id` (no Qobuz album/artist id columns),
///     so the target ids are recovered with the same `get_track` resolve
///     the Qobuz surfaces' go-to arms use; when the resolve can't deliver
///     (offline / API error / missing id) the row falls back to the LOCAL
///     destinations above, so the click always lands.
/// The window's open-album / open-artist callbacks do the final routing
/// (and the history recording).
pub(crate) fn local_row_goto(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    row: qbz_library::LocalTrack,
    to_artist: bool,
) {
    let album_key = row.album_group_key.clone();
    let artist_name = row.artist.clone();
    // Local destination (the primary route for local rows, the
    // fallback for qobuz_download ones). FnOnce — each path calls it at
    // most once, on the UI thread.
    let open_local = move |w: &AppWindow| {
        if to_artist {
            if artist_name.trim().is_empty() {
                log::debug!("[qbz-slint] go-to-artist: local row has no artist name");
                return;
            }
            w.invoke_open_artist(artist_name.into());
        } else {
            if album_key.is_empty() {
                log::debug!("[qbz-slint] go-to-album: local row has no album group key");
                return;
            }
            w.invoke_open_album(album_key.into());
        }
    };
    let qobuz_id = (row.source.as_deref() == Some("qobuz_download"))
        .then_some(row.qobuz_track_id)
        .flatten();
    match qobuz_id {
        Some(qid) if qid > 0 => {
            handle.spawn(async move {
                let resolved: Option<String> = match runtime.core().get_track(qid as u64).await {
                    Ok(track) => {
                        if to_artist {
                            track.performer.as_ref().map(|p| p.id.to_string())
                        } else {
                            track.album.as_ref().map(|a| a.id.clone())
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "[qbz-slint] go-to: get_track {qid} failed ({e}) — using the local destination"
                        );
                        None
                    }
                };
                let _ = weak.upgrade_in_event_loop(move |w| match resolved {
                    Some(qobuz_ref) if to_artist => w.invoke_open_artist(qobuz_ref.into()),
                    Some(qobuz_ref) => w.invoke_open_album(qobuz_ref.into()),
                    None => open_local(&w),
                });
            });
        }
        _ => {
            let _ = weak.upgrade_in_event_loop(move |w| open_local(&w));
        }
    }
}

/// Open a Local Library browse tab (Albums / Artists / Folders / Tracks).
///
/// Sets the active tab + switches the view, then lazily loads the tab's data
/// on first visit. Albums is the first slice (chunked grid); the other tabs
/// render their shell + a placeholder until their slices land.
pub(crate) fn navigate_local_library(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    tab: local_library::LibTab,
) {
    let tab_id = tab.tab_id().to_string();
    let _ = weak.upgrade_in_event_loop(move |w| {
        // Restore the persisted Tracks group-by before the tab derives.
        locallibrary_prefs::load(&w);
        w.global::<LocalLibraryState>().set_active_tab(tab_id.into());
        w.global::<NavState>().set_view(ContentView::LocalLibrary);
    });
    // Seed all four tab-badge counts up front (like Favorites) so the nav
    // badges show without visiting each tab.
    local_library::seed_counts(weak.clone(), handle.clone());
    // Lazy per-tab load on first visit.
    match tab {
        local_library::LibTab::Albums => {
            local_library::ensure_albums_loaded(weak, handle.clone(), image_cache);
        }
        local_library::LibTab::Folders => {
            // Tree is the default mode → load the tree roots too (the flat set
            // stays loaded so toggling to flat is instant).
            local_library::ensure_folders_loaded(weak.clone(), handle.clone(), image_cache);
            local_library::ensure_folder_tree_loaded(weak, handle.clone());
        }
        local_library::LibTab::Tracks => {
            local_library::ensure_tracks_loaded(weak, handle.clone());
        }
        local_library::LibTab::Artists => {
            local_library::ensure_artists_loaded(runtime, weak, handle.clone(), image_cache);
        }
    }
}

