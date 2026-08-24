use crate::*;

/// Open a Qobuz mix detail view (daily / weekly / fav / top) and load
/// its tracks.
pub(crate) fn navigate_mix(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    kind: String,
) {
    handle.spawn(async move {
        let kind_for_reset = kind.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            mix::reset_mix(&w, &kind_for_reset);
            w.global::<NavState>().set_view(ContentView::Mix);
        });
        let tracks = mix::load_mix(&runtime, &kind).await;
        let jobs = mix::artwork_jobs(&tracks);
        let _ = weak.upgrade_in_event_loop(move |w| {
            mix::apply_mix(&w, &kind, tracks);
        });
        artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
    });
}

pub(crate) fn navigate_playlist(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    playlist_id: String,
) {
    // Route by id namespace (D7 type guard): `local:<uuid>` ids open the
    // LOCAL detail path and can never reach the Qobuz fetch below.
    let id = match local_playlist::PlaylistRef::parse(&playlist_id) {
        Some(local_playlist::PlaylistRef::Local(id)) => {
            local_playlist::navigate(runtime, weak, handle, image_cache, id);
            return;
        }
        Some(local_playlist::PlaylistRef::Qobuz(id)) => {
            // D11.a: offline, a mixed playlist's detail renders ONLY its
            // local sidecar rows — the Qobuz membership is not enumerable
            // offline, so the API fetch below never runs.
            if offline_mode::engine().is_offline() {
                local_playlist::navigate_qobuz_offline(weak, handle, image_cache, id);
                return;
            }
            id
        }
        None => {
            log::warn!("[qbz-slint] navigate_playlist: bad id {playlist_id}");
            return;
        }
    };
    handle.spawn(async move {
        let active = playlist_id.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            playlist::reset(&w);
            sidebar::set_active(&w, &active);
            w.global::<NavState>().set_view(ContentView::Playlist);
        });
        if let Some(data) = playlist::load(&runtime, id).await {
            // Mixed rows split across loaders like the LOCAL detail:
            // Qobuz rows = http covers, local sidecar rows = file paths.
            let (http_jobs, local_jobs) = playlist::artwork_jobs(&data);
            let pid = data.id.clone();
            let owner_id = data.owner_id;
            // Seed the INTERNAL favorite heart from the library db (the open
            // handler otherwise only sets is_owner, so the heart was always
            // un-filled on open).
            let fav = pid.parse::<u64>().ok().is_some_and(|id| {
                crate::library_db::with_db(|db| db.get_favorite_playlist_ids())
                    .unwrap_or_default()
                    .contains(&id)
            });
            // Hide the Copy button if this Qobuz playlist was already copied
            // into the library (its SOURCE id is recorded). Mirrors Tauri's
            // user-scoped `qbz_copied_playlists` localStorage set.
            let copied = pid.parse::<u64>().ok().is_some_and(|id| {
                crate::library_db::with_db(|db| db.is_playlist_copied(id)).unwrap_or(false)
            });
            // Ownership = the playlist's Qobuz owner IS the current user (Tauri
            // parity: owner.id == current user id). FOLLOWED = it is in MY Qobuz
            // library (get_user_playlists) but I don't own it. Being in the
            // sidebar is a CONSEQUENCE of following, NOT the determinant (a
            // followed playlist may live in a folder, or not be rendered there
            // at all), so the authoritative source is get_user_playlists — the
            // same list the Favorites > Playlists > Followed tab is built from.
            // Owned => Delete; followed => the Follow/Unfollow toggle. The fetch
            // only runs for a non-owned playlist (your own ones skip it).
            let me = crate::library_db::current_user_id();
            let owned = me.is_some_and(|uid| uid == owner_id);
            let following = if owned || me.is_none() {
                false
            } else if let Ok(pid_u) = pid.parse::<u64>() {
                runtime
                    .core()
                    .get_user_playlists()
                    .await
                    .map(|pls| pls.iter().any(|p| p.id == pid_u))
                    .unwrap_or(false)
            } else {
                false
            };
            let _ = weak.upgrade_in_event_loop(move |w| {
                playlist::apply(&w, data);
                let st = w.global::<PlaylistState>();
                st.set_is_owner(owned);
                st.set_is_following(following);
                st.set_is_favorite(fav);
                st.set_is_copied(copied);
            });
            if !http_jobs.is_empty() {
                artwork::spawn_loads(http_jobs, weak.clone(), image_cache.clone());
            }
            if !local_jobs.is_empty() {
                artwork::spawn_local_loads(local_jobs, weak.clone(), image_cache.clone());
            }
        }
    });
}

