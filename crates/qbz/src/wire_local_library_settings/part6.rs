use crate::*;

pub(crate) fn wire_local_library_settings_part6(window: &AppWindow, _app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_window_changed(move |first, last| {
                // Windowed albums grid: dispatch covers for the reported row
                // band and evict the ones far outside it.
                if let Some(w) = weak.upgrade() {
                    local_library::albums_window_changed(&w, first, last);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_set_sort(move |sort| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>().set_albums_sort(sort);
                    local_library::derive_albums(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_set_group(move |mode| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>().set_albums_group(mode);
                    local_library::derive_albums(&w);
                }
            });
    }
    {
        // Album-identity mode (folder|metadata): the group KEY changes, so a
        // client-side derive is not enough — persist, reload the Albums set,
        // and invalidate the Artists tab (its album cache groups the same
        // way). Header dropdown + Settings row both land here.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_albums_set_id_mode(move |mode| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>().set_albums_id_mode(mode.into());
                    crate::locallibrary_prefs::save(&w);
                    local_library::invalidate_artists(&w);
                    local_library::reload_albums(w.as_weak(), handle.clone(), image_cache.clone());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_set_view(move |mode| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>().set_albums_view_mode(mode);
                    // Switching to the (non-windowed) list view needs covers
                    // the grid's window may have evicted.
                    local_library::albums_view_mode_changed(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_filter_changed(move || {
                if let Some(w) = weak.upgrade() {
                    local_library::derive_albums(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_clear_filter(move || {
                if let Some(w) = weak.upgrade() {
                    local_library::clear_album_filter(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_albums_retry(move || {
                local_library::reload_albums(weak.clone(), handle.clone(), image_cache.clone());
            });
    }
}
