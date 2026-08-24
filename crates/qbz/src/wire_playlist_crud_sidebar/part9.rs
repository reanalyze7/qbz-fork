use crate::*;

pub(crate) fn wire_playlist_crud_sidebar_part9(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Add to Mixtape/Collection (sidebar playlist context menu) — build a
        // 1-item playlist payload from the cached SidebarEntry row + the cached
        // track count, then open the global AddToMixtapeModal. Because the
        // item_type is "playlist", `open_add_to_mixtape` computes restrict=true
        // → the picker lists mixtapes only and hides the "+ Collections" chip (a
        // playlist can't live in a Collection). Mirrors the PlaylistManager path.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_add_to_mixtape(move |id| {
                use slint::Model;
                let Some(w) = weak.upgrade() else { return };
                let model = w.global::<SidebarState>().get_entries();
                let Some(row) = (0..model.row_count())
                    .filter_map(|i| model.row_data(i))
                    .find(|e| e.kind == "playlist" && e.id == id)
                else {
                    return;
                };
                let artwork = row.url1.to_string();
                let item = myqbz_add::AddItem {
                    item_type: "playlist".into(),
                    source: "qobuz".into(),
                    source_item_id: id.to_string(),
                    title: row.name.to_string(),
                    subtitle: None,
                    artwork_url: (!artwork.is_empty()).then_some(artwork),
                    year: None,
                    // SidebarEntry doesn't carry the count; pull it from the
                    // sidebar cache by id (None if unknown — it's optional).
                    track_count: id
                        .parse::<u64>()
                        .ok()
                        .and_then(sidebar::playlist_track_count)
                        .map(|n| n as i32),
                };
                open_add_to_mixtape(weak.clone(), handle.clone(), vec![item]);
            });
    }
    {
        // Edit folder (sidebar context menu) — open the folder editor.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_edit_folder(move |id| {
                let Some(w) = weak.upgrade() else { return };
                open_folder_editor(&w, id);
            });
    }
    {
        // Filter the move-to-folder menu list by a substring query.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_search_folders(move |query| {
                if let Some(w) = weak.upgrade() {
                    sidebar::search_menu_folders(&w, query.as_str());
                }
            });
    }
    {
        // Hide playlist from the sidebar (local setting), then reload.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_hide_playlist(move |id| {
                let Ok(pid) = id.parse::<u64>() else { return };
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    tokio::task::spawn_blocking(move || folders::set_hidden(pid, true))
                        .await
                        .ok();
                    load_sidebar_playlists(runtime, weak, &handle);
                });
            });
    }
    {
        // Hide folder from the sidebar (local setting), then reload.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_hide_folder(move |id| {
                let fid = id.to_string();
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    tokio::task::spawn_blocking(move || folders::set_folder_hidden(&fid, true))
                        .await
                        .ok();
                    load_sidebar_playlists(runtime, weak, &handle);
                });
            });
    }

    // === Playlist Manager actions ======================================
    wire_playlist_manager(&window, &app_runtime, &tokio_rt, &image_cache);
    wire_myqbz(&window, &app_runtime, &tokio_rt, &image_cache);
    wire_myqbz_detail(&window, &app_runtime, &tokio_rt, &image_cache);
}
