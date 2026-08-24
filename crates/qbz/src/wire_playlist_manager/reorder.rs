use crate::*;

// --- Arrow reorder (custom sort) + move-to-folder ------------------------
pub(crate) fn wire_pm_reorder(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
) {
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistManagerActions>()
            .on_move_up(move |id| {
                let Some(w) = weak.upgrade() else { return };
                let Ok(pid) = id.parse::<u64>() else { return };
                let order = playlist_manager::move_up(&w, pid);
                refresh_pm_covers(&w);
                if !order.is_empty() {
                    handle.spawn(async move {
                        tokio::task::spawn_blocking(move || folders::reorder_playlists(&order))
                            .await
                            .ok();
                    });
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistManagerActions>()
            .on_move_down(move |id| {
                let Some(w) = weak.upgrade() else { return };
                let Ok(pid) = id.parse::<u64>() else { return };
                let order = playlist_manager::move_down(&w, pid);
                refresh_pm_covers(&w);
                if !order.is_empty() {
                    handle.spawn(async move {
                        tokio::task::spawn_blocking(move || folders::reorder_playlists(&order))
                            .await
                            .ok();
                    });
                }
            });
    }
    {
        // Move a playlist into a folder ("" = root).
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistManagerActions>()
            .on_move_to_folder(move |playlist_id, folder_id| {
                let Some(w) = weak.upgrade() else { return };
                let Ok(pid) = playlist_id.parse::<u64>() else { return };
                let fid = folder_id.to_string();
                playlist_manager::move_to_folder_local(&w, pid, &fid);
                refresh_pm_covers(&w);
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    let opt = fid.clone();
                    tokio::task::spawn_blocking(move || {
                        let o = if opt.is_empty() { None } else { Some(opt.as_str()) };
                        folders::move_playlist(pid, o);
                    })
                    .await
                    .ok();
                    load_sidebar_playlists(runtime, weak, &handle);
                });
            });
    }
}
