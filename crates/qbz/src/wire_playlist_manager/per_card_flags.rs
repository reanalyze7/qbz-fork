use crate::*;

// --- Per-card playlist actions: favorite / hidden toggles ---------------
pub(crate) fn wire_pm_per_card_flags(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
) {
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistManagerActions>()
            .on_toggle_favorite(move |id| {
                let Some(w) = weak.upgrade() else { return };
                // LOCAL playlist (B3): the flag lives on its own
                // local_playlists row — the u64 settings table can't hold it.
                if local_playlist::is_local_id(id.as_str()) {
                    let value = playlist_manager::toggle_local_favorite(&w, id.as_str());
                    refresh_pm_covers(&w);
                    let lid = id.to_string();
                    handle.spawn(async move {
                        tokio::task::spawn_blocking(move || {
                            local_playlist::set_favorite_blocking(&lid, value)
                        })
                        .await
                        .ok();
                    });
                    return;
                }
                let Ok(pid) = id.parse::<u64>() else { return };
                let value = playlist_manager::toggle_favorite_local(&w, pid);
                refresh_pm_covers(&w);
                handle.spawn(async move {
                    tokio::task::spawn_blocking(move || folders::set_favorite(pid, value))
                        .await
                        .ok();
                });
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistManagerActions>()
            .on_toggle_hidden(move |id| {
                let Some(w) = weak.upgrade() else { return };
                // LOCAL playlist (B3): the flag lives on its own
                // local_playlists row; hidden locals drop from the sidebar.
                if local_playlist::is_local_id(id.as_str()) {
                    let value = playlist_manager::toggle_local_hidden(&w, id.as_str());
                    refresh_pm_covers(&w);
                    let lid = id.to_string();
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        tokio::task::spawn_blocking(move || {
                            local_playlist::set_hidden_blocking(&lid, value)
                        })
                        .await
                        .ok();
                        // The sidebar reflects hidden playlists, so refresh it.
                        load_sidebar_playlists(runtime, weak, &handle);
                    });
                    return;
                }
                let Ok(pid) = id.parse::<u64>() else { return };
                let value = playlist_manager::toggle_hidden_local(&w, pid);
                refresh_pm_covers(&w);
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    tokio::task::spawn_blocking(move || folders::set_hidden(pid, value))
                        .await
                        .ok();
                    // The sidebar reflects hidden playlists, so refresh it.
                    load_sidebar_playlists(runtime, weak, &handle);
                });
            });
    }
}
