use crate::*;

pub(crate) fn wire_create_playlist_dac_import_part7(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {

    // handler.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FavoritesActions>()
            .on_select_tab(move |id| {
                if id.as_str() == "all" {
                    nav::record(nav::NavEntry::Favorites {
                        tab: "all".to_string(),
                    });
                    if let Some(w) = weak.upgrade() {
                        update_nav_flags(&w);
                    }
                    navigate_library_all(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                    );
                    return;
                }
                let Some(tab) = favorites::FavTab::from_tab_id(id.as_str()) else {
                    // Playlists / Labels: just switch the visible tab,
                    // their content is not implemented yet.
                    if let Some(w) = weak.upgrade() {
                        w.global::<FavoritesState>().set_active_tab(id);
                    }
                    return;
                };
                // Each favorites tab is its own history page (mirrors the
                // Discover tabs) so back/forward moves between them.
                nav::record(nav::NavEntry::Favorites { tab: id.to_string() });
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                navigate_favorites(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    tab,
                    id.as_str(),
                );
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_open_album(move |id| {
                if let Some(w) = weak.upgrade() {
                    w.invoke_open_album(id);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_open_artist(move |id| {
                if let Some(w) = weak.upgrade() {
                    w.invoke_open_artist(id);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FavoritesActions>()
            .on_open_label(move |id, name| {
                let Ok(label_id) = id.parse::<u64>() else {
                    return;
                };
                let name = name.to_string();
                nav::record(nav::NavEntry::Label {
                    id: label_id,
                    name: name.clone(),
                });
                navigate_label(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    label_id,
                    name,
                );
            });
    }
}
