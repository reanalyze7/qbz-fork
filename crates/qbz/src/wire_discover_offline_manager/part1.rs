use crate::*;

pub(crate) fn wire_discover_offline_manager_part1(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        // back() — declared per the spec; the actual back chrome is the shared
        // header NavButtons (which drives nav::go_back). Wired here for any
        // future in-view trigger; routes through the same go-back path.
        let weak = window.as_weak();
        let app_runtime_bl = app_runtime.clone();
        let bl_handle = tokio_rt.handle().clone();
        let image_cache_bl = image_cache.clone();
        window.global::<BlacklistActions>().on_back(move || {
            if let Some((entry, scroll)) = nav::go_back() {
                let weak2 = weak.clone();
                arm_scroll_restore(&weak2, &entry, scroll);
                apply_entry(
                    entry,
                    &app_runtime_bl,
                    &weak2,
                    &bl_handle,
                    &image_cache_bl,
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            }
        });
    }
    {
        let weak = window.as_weak();
        let bl_runtime_a = app_runtime.clone();
        let bl_handle_a = tokio_rt.handle().clone();
        let bl_image_cache_a = image_cache.clone();
        window
            .global::<BlacklistActions>()
            .on_artist_select(move |id| {
                let artist_id = id.to_string();
                nav::record(nav::NavEntry::Artist(artist_id.clone()));
                navigate_artist(
                    bl_runtime_a.clone(),
                    weak.clone(),
                    &bl_handle_a,
                    bl_image_cache_a.clone(),
                    artist_id,
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_toggle_enabled(move || {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::toggle_enabled(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window.global::<BlacklistActions>().on_remove(move |id| {
            if let Some(w) = weak.upgrade() {
                blacklist_manager::remove(&w, id);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<BlacklistActions>().on_clear_all(move || {
            if let Some(w) = weak.upgrade() {
                blacklist_manager::clear_all(&w);
            }
        });
    }
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_search_changed(move |q| {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::search_changed(&w, q.to_string());
                }
            });
    }
    // --- Album blacklist callbacks ---
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_set_tab(move |tab| {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::set_tab(&w, tab);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_block_album(move |id, title, artist, cover| {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::block_album(
                        &w,
                        id.to_string(),
                        title.to_string(),
                        artist.to_string(),
                        cover.to_string(),
                    );
                }
            });
    }
}
