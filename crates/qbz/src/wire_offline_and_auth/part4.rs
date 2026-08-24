use crate::*;
use crate::navigate_album_artist::nav_statics::SEARCH_DEBOUNCE;

pub(crate) fn wire_offline_and_auth_part4(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Submit search (Enter): record history and show the results page.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<SearchActions>().on_submit(move |query| {
            let q = query.trim().to_string();
            if q.len() < 2 {
                return;
            }
            SEARCH_DEBOUNCE.with(|t| t.stop());
            nav::push_or_replace_search(q.clone());
            navigate_search(runtime.clone(), weak.clone(), &handle, image_cache.clone(), q);
            if let Some(w) = weak.upgrade() {
                // Enter -> results page: dismiss the live dropdown and always
                // land on Search > All (never a lingering per-type tab).
                let st = w.global::<SearchState>();
                st.set_cortinilla_open(false);
                st.set_tab(0);
                update_nav_flags(&w);
            }
        });
    }
}
