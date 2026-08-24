use crate::*;
use crate::navigate_album_artist::nav_statics::SEARCH_DEBOUNCE;

pub(crate) fn wire_search_part5(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Cortinilla: Enter with nothing highlighted — run a full search-all on the
    // current live query (same path as submit) and dismiss the dropdown.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<SearchActions>()
            .on_cortinilla_search_all(move || {
                let Some(w) = weak.upgrade() else { return };
                let q = w
                    .global::<SearchState>()
                    .get_cortinilla_query()
                    .trim()
                    .to_string();
                if q.chars().count() < 2 {
                    return;
                }
                let st = w.global::<SearchState>();
                st.set_cortinilla_open(false);
                // Activating the cortinilla's Enter affordance clears the input
                // too (consistent with row-click / View-more), so it can't
                // re-invoke the dropdown over the results page.
                st.set_header_search_text("".into());
                // Enter always lands on Search > All, never a per-type tab.
                st.set_tab(0);
                SEARCH_DEBOUNCE.with(|t| t.stop());
                nav::push_or_replace_search(q.clone());
                navigate_search(runtime.clone(), weak.clone(), &handle, image_cache.clone(), q);
                update_nav_flags(&w);
            });
    }
}
