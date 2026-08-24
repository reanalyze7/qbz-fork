use crate::*;

pub(crate) fn wire_discover_offline_manager_part12(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Local Library — in-view tab bar (select-tab) + the gear button
    // (open-settings -> Settings > Local Library). Same per-tab history
    // pattern as Favorites.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_select_tab(move |tab_id| {
                if let Some(tab) = local_library::LibTab::from_tab_id(tab_id.as_str()) {
                    nav::record(nav::NavEntry::LocalLibrary {
                        tab: tab.tab_id().to_string(),
                    });
                    if let Some(w) = weak.upgrade() {
                        update_nav_flags(&w);
                    }
                    navigate_local_library(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        tab,
                    );
                }
            });
    }
}
