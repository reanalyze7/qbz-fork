use crate::*;

pub(crate) fn wire_playlist_crud_sidebar_part3(window: &AppWindow, _app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {

    // Playlist in-page track search (client-side filter).
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistActions>()
            .on_search(move |query| {
                if let Some(w) = weak.upgrade() {
                    playlist::filter_tracks(&w, query.as_str());
                }
            });
    }
    // "Hi-Res only" filter — Rust-side because the list is virtualized.
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistActions>()
            .on_set_hires_only(move |on| {
                if let Some(w) = weak.upgrade() {
                    playlist::set_hires_only(&w, on);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistActions>()
            .on_set_sort(move |field| {
                let Some(w) = weak.upgrade() else { return; };
                playlist::set_sort(&w, field.as_str());
                // Entering custom: load (or seed) the local order, then
                // re-render. Off-thread (opens library.db).
                if field.as_str() == "custom" {
                    let pid = w.global::<PlaylistState>().get_id().to_string();
                    if let Ok(pid) = pid.parse::<u64>() {
                        // Seed keys carry (id, is_local) — Qobuz rows then
                        // local sidecar rows (Tauri parity).
                        let seed = playlist::custom_seed_keys();
                        let weak = weak.clone();
                        handle.spawn(async move {
                            let orders = tokio::task::spawn_blocking(move || {
                                playlist::load_or_init_custom(pid, seed)
                            })
                            .await
                            .unwrap_or_default();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                playlist::apply_custom_order(&w, orders);
                            });
                        });
                    }
                }
            });
    }
    // Drag-reorder within the custom-order track list (issue #589): the
    // drop commits ONE from->to move. Routes like the move-up/move-down
    // chevron arms: LOCAL playlists write the repo position order directly
    // (repo::reorder, B2); Qobuz playlists rebuild the custom-order sidecar
    // optimistically and persist the full order off-thread.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistActions>()
            .on_reorder_track(move |from, to| {
                let Some(w) = weak.upgrade() else { return; };
                if from < 0 || to < 0 || to == from || to == from + 1 {
                    return;
                }
                let (from, to) = (from as usize, to as usize);
                let pid = w.global::<PlaylistState>().get_id().to_string();
                if local_playlist::is_local_id(&pid) {
                    local_playlist::reorder_row(&w, &handle, from, to);
                } else {
                    let orders = playlist::reorder_track(&w, from, to);
                    if !orders.is_empty() {
                        if let Ok(pid) = pid.parse::<u64>() {
                            handle.spawn(async move {
                                tokio::task::spawn_blocking(move || {
                                    playlist::persist_custom(pid, orders);
                                })
                                .await
                                .ok();
                            });
                        }
                    }
                }
            });
    }

    // Edit playlist (rename / delete).
    {
        let weak = window.as_weak();
        window
            .global::<EditPlaylistActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<EditPlaylistState>().set_open(false);
                }
            });
    }
}
