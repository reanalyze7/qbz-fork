use crate::*;

pub(crate) fn wire_playlist_browse_picker_part1(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<EphemeralPlayChoiceActions>()
            .on_enqueue(move || {
                if let Some(w) = weak.upgrade() {
                    let s = w.global::<EphemeralPlayChoiceState>();
                    let kind = s.get_intent_kind().to_string();
                    let arg = s.get_intent_arg().to_string();
                    s.set_open(false);
                    playback::ephemeral_enqueue(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        kind,
                        arg,
                    );
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<EphemeralPlayChoiceActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<EphemeralPlayChoiceState>().set_open(false);
                }
            });
    }

    // Restore a previously-open ephemeral folder (re-scans the path; does NOT
    // switch the landing view). Runs once at startup.
    local_library::rehydrate_ephemeral(window.as_weak(), tokio_rt.handle().clone());

    // ---- Artists tab actions ----
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_artists_search(move |_query| {
                // Query is two-way bound to artists-search; re-derive in place.
                if let Some(w) = weak.upgrade() {
                    local_library::derive_artists(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_artists_select(move |name| {
                local_library::select_local_artist(
                    weak.clone(),
                    handle.clone(),
                    image_cache.clone(),
                    name.to_string(),
                );
            });
    }

    // Discover "View all" — open the full-list page for a section,
    // recording it as a history entry (mirrors the favorites branch
    // of on_header_menu_navigate).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.on_discover_view_all(move |endpoint, title| {
            nav::record(nav::NavEntry::DiscoverBrowse {
                endpoint: endpoint.to_string(),
                title: title.to_string(),
            });
            if let Some(w) = weak.upgrade() {
                update_nav_flags(&w);
            }
            discover_browse::navigate(
                runtime.clone(),
                weak.clone(),
                &handle,
                image_cache.clone(),
                endpoint.to_string(),
                title.to_string(),
                current_genre_filter(),
            );
        });
    }

    // Discover "View all" pagination — load the next album page when
    // the grid scrolls near the bottom.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<DiscoverBrowseActions>()
            .on_load_more(move || {
                discover_browse::load_more(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    current_genre_filter(),
                );
            });
    }
}
