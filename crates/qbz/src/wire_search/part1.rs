use crate::*;

pub(crate) fn wire_search_part1(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Live search: debounce 300 ms, minimum 2 characters. Does not record
    // history (per-keystroke entries would pollute the back stack).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<SearchActions>().on_live(move |query| {
            let q = query.trim().to_string();
            // chars().count(): the >= 2 gate is on grapheme-ish length, not
            // bytes, so a 2-char multibyte query (e.g. CJK) is not rejected.
            if q.chars().count() < 2 {
                SEARCH_DEBOUNCE.with(|t| t.stop());
                // Below the threshold — close the cortinilla so a backspaced
                // query does not leave a stale dropdown open.
                if let Some(w) = weak.upgrade() {
                    w.global::<SearchState>().set_cortinilla_open(false);
                }
                return;
            }

            // --- Cortinilla (live dropdown), only when the module is ON (D5) ---
            // The results-page debounce below is untouched; the cortinilla is a
            // separate, additive surface gated on the kill switch.
            if crate::search_service::is_enabled() {
                spawn_cortinilla_live_search(&weak, &runtime, &handle, &image_cache, &q);
            }

            // --- Results page LIVE search — ONLY when the module is OFF --------
            // When Intelligent Search is ON, the cortinilla above is the live
            // preview; typing must NOT auto-navigate to the results page. The
            // 300 ms debounce-navigate would otherwise hijack navigation — a
            // pending fire lands on the results page ~300 ms after the last
            // keystroke and overrides wherever the user just went (e.g. a
            // cortinilla row-click), so "I can't navigate anywhere, it takes me
            // to the result". Enter (on_submit) still navigates there. When the
            // module is OFF, keep the Phase-1 live-results behavior unchanged.
            if crate::search_service::is_enabled() {
                SEARCH_DEBOUNCE.with(|t| t.stop());
                return;
            }
            // --- Results page (module OFF): debounce 300 ms, then full search ---
            let runtime = runtime.clone();
            let weak = weak.clone();
            let handle = handle.clone();
            let image_cache = image_cache.clone();
            SEARCH_DEBOUNCE.with(|t| {
                t.start(
                    slint::TimerMode::SingleShot,
                    std::time::Duration::from_millis(300),
                    move || {
                        // Record (or replace) the Search history entry so
                        // back/forward returns to this search instead of
                        // skipping past it.
                        nav::push_or_replace_search(q.clone());
                        navigate_search(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            q.clone(),
                        );
                        if let Some(w) = weak.upgrade() {
                            update_nav_flags(&w);
                        }
                    },
                );
            });
        });
    }
}
