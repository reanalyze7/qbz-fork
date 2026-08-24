use crate::*;

/// Warm the shared favorite-track / favorite-album / per-artist-library /
/// followed-artist caches so track rows, album headers, the ArtistPage
/// catalog toggle, and the Pinned carousel's follow chip show correct state
/// from first paint. Fire-and-forget: each warm is its own spawned task,
/// skipped while offline (the disk seed from session activation is the
/// truth there).
pub(crate) fn es_warm_caches(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
) {
    // Warm the shared favorite-track cache so track rows can show the
    // correct heart state from their first paint (album / artist / search
    // / playlist / mix / favorites / queue all read it). The disk seed
    // already ran at session activation (fav_cache::init_for_user); this
    // refreshes from the network and writes the fresh set back — skipped
    // while offline, where the disk seed is the truth.
    {
        let runtime = runtime.clone();
        tokio::spawn(async move {
            if crate::offline_mode::engine().is_offline() {
                return;
            }
            match runtime.core().favorite_track_ids().await {
                Ok(ids) => {
                    // set_all mirrors to disk (blocking rusqlite) — keep it
                    // off the async worker.
                    let _ = tokio::task::spawn_blocking(move || fav_cache::set_all(ids)).await;
                }
                Err(e) => log::warn!("[qbz-slint] favorite cache load failed: {e}"),
            }
        });
    }

    // Same for favorite ALBUMS — seeds fav_cache so the album header heart is
    // correct from first open without visiting the Favorites view.
    {
        let runtime = runtime.clone();
        tokio::spawn(async move {
            if crate::offline_mode::engine().is_offline() {
                return;
            }
            let ids = favorites::favorite_album_ids(&runtime).await;
            let _ = tokio::task::spawn_blocking(move || fav_cache::set_all_albums(ids)).await;
        });
    }

    // Seed the per-artist library index so the ArtistPage catalog/library toggle
    // can decide (O(1)) whether the user has items for that artist. Favorites-
    // only (tracks + albums), once per session, off the UI thread.
    {
        let runtime = runtime.clone();
        tokio::spawn(async move {
            if crate::offline_mode::engine().is_offline() {
                return;
            }
            crate::library_by_artist::seed(&runtime).await;
        });
    }

    // Same for followed ARTISTS — the Pinned carousel's artist follow chip
    // seeds from fav_cache at build time (its only build-time consumer), and
    // the pinned model is built BEFORE this warm lands: re-seed any already-
    // built artist rows once the fresh set arrives (walk > rebuild_pinned: no
    // model swap, no artwork-job re-dispatch, no flicker).
    {
        let runtime = runtime.clone();
        let weak = weak.clone();
        tokio::spawn(async move {
            if crate::offline_mode::engine().is_offline() {
                return;
            }
            match runtime.core().favorite_artist_ids().await {
                Ok(ids) => {
                    let _ =
                        tokio::task::spawn_blocking(move || fav_cache::set_all_artists(ids)).await;
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        let pm = w.global::<PinnedState>().get_items();
                        for i in 0..pm.row_count() {
                            if let Some(mut it) = pm.row_data(i) {
                                if it.kind == "artist" {
                                    let following = it
                                        .artist
                                        .id
                                        .parse::<u64>()
                                        .map(|id| fav_cache::is_artist_favorite(id))
                                        .unwrap_or(false);
                                    if it.artist.following != following {
                                        it.artist.following = following;
                                        pm.set_row_data(i, it);
                                    }
                                }
                            }
                        }
                        // External-reco artist rows (Discover > Recommendations):
                        // same in-place re-seed — the rows may already be painted
                        // from the results blob before this warm lands (their
                        // build-time fav_cache seed was stale/empty then).
                        let reco = w.global::<ExternalRecoState>();
                        for model in [
                            reco.get_rec_artists_common(),
                            reco.get_rec_artists_recent(),
                            reco.get_top_artists(),
                        ] {
                            for i in 0..model.row_count() {
                                if let Some(mut it) = model.row_data(i) {
                                    let following = it
                                        .id
                                        .parse::<u64>()
                                        .map(|id| fav_cache::is_artist_favorite(id))
                                        .unwrap_or(false);
                                    if it.following != following {
                                        it.following = following;
                                        model.set_row_data(i, it);
                                    }
                                }
                            }
                        }
                    });
                }
                Err(e) => log::warn!("[qbz-slint] favorite artists warm failed: {e}"),
            }
        });
    }
}
