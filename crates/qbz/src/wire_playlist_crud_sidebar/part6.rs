use crate::*;

pub(crate) fn wire_playlist_crud_sidebar_part6(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Sidebar playlists — open a row, or create a new playlist.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<SidebarActions>()
            .on_open_playlist(move |id| {
                nav::record(nav::NavEntry::Playlist(id.to_string()));
                navigate_playlist(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    id.to_string(),
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    {
        // Populate the collapsed-sidebar folder flyout's playlist list.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_load_folder_popup(move |folder_id| {
                if let Some(w) = weak.upgrade() {
                    sidebar::load_folder_popup(&w, folder_id.as_str());
                }
            });
    }
    {
        // The "+" shortcut now opens the unified picker (PlaylistAddModal)
        // pre-deployed to its inline create row, instead of the retired
        // CreatePlaylistModal (spec §2/§3): same `open_for_ids` entry point
        // as every other "Add to playlist" trigger, just with an empty
        // id list (create-only, no pending tracks — `on_create_and_add`
        // already handles 0 carried ids as a no-op add, matching the old
        // create-without-adding shortcut).
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_create_playlist(move || {
                if let Some(w) = weak.upgrade() {
                    playlist_picker::open_for_ids(&w, runtime.clone(), &handle, Vec::new(), false);
                    let picker = w.global::<PlaylistPickerState>();
                    picker.set_creating_open(true);
                    picker.set_create_name("".into());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<CreateFolderActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<CreateFolderState>().set_open(false);
                }
            });
    }
    {
        // Create a folder, then refresh the sidebar.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<CreateFolderActions>()
            .on_submit(move || {
                let Some(w) = weak.upgrade() else { return; };
                let name = w.global::<CreateFolderState>().get_name().to_string();
                if name.trim().is_empty() || w.global::<CreateFolderState>().get_creating() {
                    return;
                }
                w.global::<CreateFolderState>().set_creating(true);
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    let nm = name.trim().to_string();
                    tokio::task::spawn_blocking(move || {
                        folders::create_folder(&nm);
                    })
                    .await
                    .ok();
                    let r2 = runtime.clone();
                    let w2 = weak.clone();
                    let h2 = handle.clone();
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        w.global::<CreateFolderState>().set_creating(false);
                        w.global::<CreateFolderState>().set_open(false);
                        load_sidebar_playlists(r2, w2, &h2);
                    });
                });
            });
    }
}
