use crate::*;

/// Toggle the INTERNAL library-favorite flag of a playlist (by id), reading
/// the authoritative current state from the DB so a card (which can't know it)
/// flips the right way. `is_open` mirrors the open detail's PlaylistState.
pub(crate) fn playlist_toggle_favorite_by_id(
    handle: tokio::runtime::Handle,
    weak: slint::Weak<AppWindow>,
    pid: u64,
    is_open: bool,
) {
    handle.spawn_blocking(move || {
        let currently = crate::library_db::with_db(|db| db.get_favorite_playlist_ids())
            .unwrap_or_default()
            .contains(&pid);
        let new_fav = !currently;
        let ok = crate::library_db::with_db(|db| db.set_playlist_favorite(pid, new_fav));
        if ok.is_none() {
            log::error!("[qbz-slint] toggle playlist {pid} favorite failed");
            return;
        }
        if is_open {
            let _ = weak.upgrade_in_event_loop(move |w| {
                w.global::<PlaylistState>().set_is_favorite(new_fav);
            });
        }
    });
}

/// Toggle a track favorite by its REAL Qobuz id: offline guard (read-only
/// hearts, spec 4.3), optimistic flip across the visible rows + the shared
/// fav cache, then the network add/remove with rollback on failure. Shared
/// by the Qobuz-surface `("track","favorite")` media-action arm and the
/// library-surface favorite entry (qobuz_download rows resolve their
/// `qobuz_track_id` first — never the local row id, which is Tauri's latent
/// "Add to Library" bug; LocalLibrary track-menu spec §3.2). UI-thread only
/// (upgrades `weak` directly).
pub(crate) fn toggle_track_favorite(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    id: String,
) {
    if offline_mode::engine().is_offline() {
        if let Some(w) = weak.upgrade() {
            toast::info(&w, "Not available offline");
        }
        return;
    }
    // Toggle (not just add): read the cached state, flip it optimistically
    // across every visible track model + the shared cache, then add/remove
    // on the network.
    let was_fav = fav_cache::is_favorite(&id);
    let make_fav = !was_fav;
    if let Ok(track_id) = id.parse::<u64>() {
        fav_cache::set(track_id, make_fav);
    }
    if let Some(w) = weak.upgrade() {
        set_row_favorite(&w, &id, make_fav);
    }
    handle.spawn(async move {
        let res = if make_fav {
            runtime.core().add_favorite("track", &id).await
        } else {
            runtime.core().remove_favorite("track", &id).await
        };
        // reco: log a favorite ADD on success (skip removes/failures) for taste
        // scoring; blocking SQLite off the async path.
        if make_fav && res.is_ok() {
            if let Ok(tid) = id.parse::<u64>() {
                tokio::task::spawn_blocking(move || {
                    crate::reco::log_favorite_track(tid, None, None)
                });
            }
        }
        if let Err(e) = res {
            log::error!("[qbz-slint] toggle track favorite failed: {e}");
            // Roll the optimistic change back on failure.
            if let Ok(tid) = id.parse::<u64>() {
                fav_cache::set(tid, was_fav);
            }
            let _ = weak.upgrade_in_event_loop(move |w| {
                set_row_favorite(&w, &id, was_fav);
            });
        }
    });
}

/// Look up the display name of an "Add to Mixtape/Collection" picker row by id
/// (for the post-add toast). Returns "" if not found.
pub(crate) fn myqbz_add_row_name(window: &AppWindow, collection_id: &str) -> String {
    use slint::Model;
    let model = window.global::<MyQbzAddState>().get_rows();
    (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .find(|r| r.id == collection_id)
        .map(|r| r.name.to_string())
        .unwrap_or_default()
}

