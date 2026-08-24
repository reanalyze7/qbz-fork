use crate::*;

pub(crate) fn wire_playlist_crud_sidebar_part8(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // Refresh — re-fetch the playlist list from the network.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_refresh_playlists(move || {
                load_sidebar_playlists(runtime.clone(), weak.clone(), &handle);
            });
    }
    {
        // Manage playlists — open the full Playlist Manager view.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<SidebarActions>()
            .on_manage_playlists(move || {
                nav::record(nav::NavEntry::PlaylistManager);
                playlist_manager::navigate(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    {
        // Import playlist — open the importer modal fully reset, with the
        // folder dropdown rebuilt from the current sidebar folder list.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_import_playlist(move || {
                if let Some(w) = weak.upgrade() {
                    playlist_import::open(&w);
                }
            });
    }
    {
        // Edit playlist (sidebar context menu) — open the edit-playlist
        // modal, prefilled from the cached name + description.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_edit_playlist(move |id| {
                let Some(w) = weak.upgrade() else { return };
                let es = w.global::<EditPlaylistState>();
                // LOCAL playlist row — prefill from the sidebar's local
                // cache (name/description/offline-only).
                if local_playlist::is_local_id(id.as_str()) {
                    let (name, description, offline_only) =
                        sidebar::local_playlist_meta(id.as_str())
                            .unwrap_or_else(|| (id.to_string(), String::new(), false));
                    es.set_id(id);
                    es.set_name(name.into());
                    es.set_description(description.into());
                    es.set_is_local(true);
                    es.set_offline_only(offline_only);
                    es.set_busy(false);
                    es.set_open(true);
                    return;
                }
                let (name, description) = id
                    .parse::<u64>()
                    .ok()
                    .and_then(sidebar::playlist_name_desc)
                    .unwrap_or_else(|| (id.to_string(), String::new()));
                es.set_id(id);
                es.set_name(name.into());
                es.set_description(description.into());
                es.set_is_local(false);
                es.set_offline_only(false);
                es.set_busy(false);
                es.set_open(true);
            });
    }
}
