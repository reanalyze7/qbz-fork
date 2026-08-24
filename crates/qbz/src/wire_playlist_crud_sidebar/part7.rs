use crate::*;

pub(crate) fn wire_playlist_crud_sidebar_part7(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Toggle a folder's expanded state (cheap, rebuilds from cache).
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_toggle_folder(move |id| {
                if let Some(w) = weak.upgrade() {
                    sidebar::toggle_folder(&w, id.as_str());
                    refresh_sidebar_covers(&w);
                }
            });
    }
    {
        // Open the create-folder modal.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_create_folder(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<CreateFolderState>().set_name("".into());
                    w.global::<CreateFolderState>().set_creating(false);
                    w.global::<CreateFolderState>().set_open(true);
                }
            });
    }
    {
        // Delete a folder (its playlists fall back to root via the
        // library DB's ON DELETE SET NULL), then reload the sidebar.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_delete_folder(move |id| {
                let id = id.to_string();
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    let fid = id.clone();
                    tokio::task::spawn_blocking(move || folders::delete_folder(&fid))
                        .await
                        .ok();
                    load_sidebar_playlists(runtime, weak, &handle);
                });
            });
    }
    {
        // Move a playlist into a folder ("" = root). Optimistic local
        // rebuild + a DB write.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_move_playlist(move |playlist_id, folder_id| {
                let Some(w) = weak.upgrade() else { return; };
                let fid = folder_id.to_string();
                // LOCAL playlists (`local:<uuid>`) persist into the
                // local_playlists.folder_id column; Qobuz ones into
                // playlist_settings. Both join the SAME shared folders.
                if local_playlist::is_local_id(&playlist_id) {
                    let id = playlist_id.to_string();
                    sidebar::move_local_playlist_local(&w, &id, &fid);
                    refresh_sidebar_covers(&w);
                    handle.spawn(async move {
                        tokio::task::spawn_blocking(move || {
                            let opt = if fid.is_empty() { None } else { Some(fid.as_str()) };
                            folders::move_local_playlist(&id, opt);
                        })
                        .await
                        .ok();
                    });
                    return;
                }
                let Ok(pid) = playlist_id.parse::<u64>() else { return; };
                sidebar::move_playlist_local(&w, pid, &fid);
                refresh_sidebar_covers(&w);
                handle.spawn(async move {
                    tokio::task::spawn_blocking(move || {
                        let opt = if fid.is_empty() { None } else { Some(fid.as_str()) };
                        folders::move_playlist(pid, opt);
                    })
                    .await
                    .ok();
                });
            });
    }
    {
        // Pick a playlist sort option (name/recent/tracks/playcount/custom).
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_set_sort(move |option| {
                if let Some(w) = weak.upgrade() {
                    sidebar::set_sort(&w, option.as_str());
                    refresh_sidebar_covers(&w);
                }
            });
    }
    {
        // Re-run the playlist-name filter as the search input edits.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_search_changed(move |query| {
                if let Some(w) = weak.upgrade() {
                    sidebar::set_search(&w, query.as_str());
                    refresh_sidebar_covers(&w);
                }
            });
    }
}
