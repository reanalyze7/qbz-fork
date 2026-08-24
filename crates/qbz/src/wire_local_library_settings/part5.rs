use crate::*;

pub(crate) fn wire_local_library_settings_part5(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Add the whole local album to a Mixtape/Collection. Builds the
        // `album` payload (source "local", no artwork_url — 1:1 PSD) from the
        // LocalAlbumState header + the current version's track count.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<LocalAlbumActions>().on_add_to_mixtape(move || {
            if let Some(w) = weak.upgrade() {
                let st = w.global::<LocalAlbumState>();
                let id = st.get_id().to_string();
                if id.is_empty() {
                    return;
                }
                let tracks = local_library::current_album_version_tracks(&w);
                let item = myqbz_add::AddItem {
                    item_type: "album".into(),
                    source: "local".into(),
                    source_item_id: id,
                    title: st.get_title().to_string(),
                    subtitle: {
                        let a = st.get_artist().to_string();
                        (!a.is_empty()).then_some(a)
                    },
                    artwork_url: None,
                    year: None,
                    track_count: (!tracks.is_empty()).then_some(tracks.len() as i32),
                };
                open_add_to_mixtape(weak.clone(), handle.clone(), vec![item]);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<LocalAlbumActions>().on_select_version(move |i| {
            if let Some(w) = weak.upgrade() {
                local_library::apply_album_version(&w, i);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<LocalAlbumActions>().on_search(move |q| {
            local_library::search_album(weak.clone(), q.to_string());
        });
    }
    {
        // Per-disc "Disc N" header ⋯ menu (local album) — scoped to that disc's
        // tracks only, resolved from the open version's track cache. Reuses the
        // SAME local queue ops as the header play-all / shuffle buttons
        // (play_local_tracks, shuffle flag) and the per-row menu's
        // enqueue_local_tracks, just over the disc subset.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalAlbumActions>()
            .on_disc_action(move |disc, action| {
                let Some(w) = weak.upgrade() else { return };
                let tracks = local_library::current_album_disc_tracks(&w, disc);
                if tracks.is_empty() {
                    return;
                }
                match action.as_str() {
                    "play" => playback::play_local_tracks(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        tracks,
                        0,
                        false,
                    ),
                    "shuffle" => playback::play_local_tracks(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        tracks,
                        0,
                        true,
                    ),
                    "queue" => playback::enqueue_local_tracks(
                        runtime.clone(),
                        handle.clone(),
                        tracks,
                        false,
                    ),
                    "play-next" => playback::enqueue_local_tracks(
                        runtime.clone(),
                        handle.clone(),
                        tracks,
                        true,
                    ),
                    other => {
                        log::warn!("[qbz-slint] local disc-action: unknown action {other}");
                    }
                }
            });
    }

    // Local Library — Albums tab controls (search / sort re-query page 1;
    // load-more pages on scroll; retry) + the shared AlbumCollectionView's
    // open / per-card actions (album-detail + playback land with later slices).
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_search(move |_query| {
                // Two-way bound to albums-search; re-derive in memory (full-load).
                if let Some(w) = weak.upgrade() {
                    local_library::derive_albums(&w);
                }
            });
    }
}
