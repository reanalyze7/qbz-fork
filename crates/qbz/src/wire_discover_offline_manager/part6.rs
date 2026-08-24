use crate::*;

pub(crate) fn wire_discover_offline_manager_part6(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {

    // Qobuz Playlists category filter (multi-select, client-side). Toggling /
    // clearing a tag re-filters the cached playlists row and re-fires the
    // artwork for the new (filtered) positions — no re-fetch.
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<HomeActions>()
            .on_toggle_playlist_tag(move |slug| {
                if let Some(w) = weak.upgrade() {
                    let jobs = home::toggle_playlist_tag(&w, slug.as_str());
                    artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<HomeActions>()
            .on_clear_playlist_tags(move || {
                if let Some(w) = weak.upgrade() {
                    let jobs = home::clear_playlist_tags(&w);
                    artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                }
            });
    }

    // Discover section configurator (Slice 5) — gear opens the modal; toggle /
    // move / reset mutate the per-user prefs, persist, and re-render the active
    // tab from the cache (no refetch). The mutation handlers re-fire artwork for
    // newly-shown Home/Editor album sections, mirroring on_select_tab.
    {
        let weak = window.as_weak();
        window
            .global::<DiscoverActions>()
            .on_open_configurator(move || {
                if let Some(w) = weak.upgrade() {
                    discover_prefs::on_open_configurator(&w);
                }
            });
    }
    // Recommendations-tab cache controls (unique to this tab).
    {
        let weak = window.as_weak();
        window
            .global::<ExternalRecoActions>()
            .on_set_cache_ttl(move |index| {
                if let Some(w) = weak.upgrade() {
                    discover_prefs::set_reco_cache_ttl_index(&w, index);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<ExternalRecoActions>()
            .on_refresh_now(move || {
                external_reco::force_reload(&runtime, &weak, &handle, &image_cache);
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<DiscoverActions>()
            .on_close_configurator(move || {
                if let Some(w) = weak.upgrade() {
                    discover_prefs::on_close_configurator(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<DiscoverActions>()
            .on_toggle_section(move |tab, id| {
                if let Some(w) = weak.upgrade() {
                    discover_prefs::on_toggle(&w, tab.as_str(), id.as_str(), &image_cache);
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<DiscoverActions>()
            .on_move_section(move |tab, id, dir| {
                if let Some(w) = weak.upgrade() {
                    discover_prefs::on_move(&w, tab.as_str(), id.as_str(), dir, &image_cache);
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<DiscoverActions>()
            .on_reset_tab(move |tab| {
                if let Some(w) = weak.upgrade() {
                    discover_prefs::on_reset(&w, tab.as_str(), &image_cache);
                }
            });
    }
}
