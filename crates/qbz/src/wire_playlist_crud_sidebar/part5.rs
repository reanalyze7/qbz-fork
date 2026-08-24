use crate::*;

pub(crate) fn wire_playlist_crud_sidebar_part5(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Delete the playlist, then navigate back + refresh the sidebar.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<EditPlaylistActions>()
            .on_delete(move || {
                let Some(w) = weak.upgrade() else { return; };
                let id = w.global::<EditPlaylistState>().get_id().to_string();
                log::info!(
                    "[playlist-delete] requested: id='{id}' is_local={}",
                    local_playlist::is_local_id(&id)
                );
                // LOCAL playlist — delete the library.db entity (cascades
                // its membership rows), then back + sidebar reload.
                if local_playlist::is_local_id(&id) {
                    w.global::<EditPlaylistState>().set_busy(true);
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let lid = id.clone();
                        let nav_id = id.clone();
                        let ok = tokio::task::spawn_blocking(move || {
                            local_playlist::delete_blocking(&lid)
                        })
                        .await
                        .unwrap_or(false);
                        log::info!("[playlist-delete] local delete result -> {ok}");
                        let r2 = runtime.clone();
                        let w2 = weak.clone();
                        let h2 = handle.clone();
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            w.global::<EditPlaylistState>().set_busy(false);
                            if ok {
                                w.global::<EditPlaylistState>().set_open(false);
                                load_sidebar_playlists(r2, w2, &h2);
                                // §3: only step back when viewing THIS playlist's
                                // detail; otherwise stay on the invoking surface
                                // (the sidebar refresh above drops the row).
                                let on_detail = w.global::<NavState>().get_view() == ContentView::Playlist
                                    && w.global::<PlaylistState>().get_id().to_string() == nav_id;
                                if on_detail {
                                    w.global::<NavState>().invoke_request_back();
                                }
                            }
                        });
                    });
                    return;
                }
                let Ok(pid) = id.parse::<u64>() else {
                    log::warn!("[playlist-delete] non-numeric Qobuz id '{id}' — aborting");
                    return;
                };
                w.global::<EditPlaylistState>().set_busy(true);
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                let id_for_nav = id.clone();
                handle.clone().spawn(async move {
                    // Re-derive ownership server-side — the modal opens from
                    // surfaces (sidebar context menu / manager) that don't carry
                    // the owner flag, so never trust the UI here. OWNED => delete;
                    // FOLLOWED/subscribed (not owned) => unsubscribe. Qobuz's
                    // playlist/delete returns 200 but NO-OPS on a playlist you
                    // don't own (the "deleted ok but it stays" bug), so a followed
                    // playlist MUST go through unsubscribe.
                    let me = crate::library_db::current_user_id();
                    let owned = match runtime.core().get_playlist(pid).await {
                        Ok(p) => me.is_some_and(|uid| uid == p.owner.id),
                        Err(e) => {
                            log::warn!(
                                "[playlist-delete] {pid} owner check failed ({e}); treating as not-owned"
                            );
                            false
                        }
                    };
                    let res = if owned {
                        log::info!("[playlist-delete] {pid} OWNED -> delete");
                        runtime.core().delete_playlist(pid).await
                    } else {
                        log::info!("[playlist-delete] {pid} FOLLOWED -> unsubscribe");
                        runtime.core().unsubscribe_playlist(pid).await
                    };
                    match res {
                        Ok(()) => {
                            log::info!("[playlist-delete] {pid} removed ok (owned={owned})");
                            let r2 = runtime.clone();
                            let w2 = weak.clone();
                            let h2 = handle.clone();
                            let nav_id = id_for_nav.clone();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                w.global::<EditPlaylistState>().set_busy(false);
                                w.global::<EditPlaylistState>().set_open(false);
                                load_sidebar_playlists(r2, w2, &h2);
                                // §3: only step back when viewing THIS playlist's
                                // detail; else stay on the invoking surface (the
                                // sidebar refresh above drops the row).
                                let on_detail = w.global::<NavState>().get_view() == ContentView::Playlist
                                    && w.global::<PlaylistState>().get_id().to_string() == nav_id;
                                if on_detail {
                                    w.global::<NavState>().invoke_request_back();
                                }
                            });
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] remove playlist failed: {e}");
                            let _ = weak.upgrade_in_event_loop(|w| {
                                w.global::<EditPlaylistState>().set_busy(false);
                            });
                        }
                    }
                });
            });
    }
}
