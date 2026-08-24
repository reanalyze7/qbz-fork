use crate::*;

// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this fn wraps ONE
// original fn main() statement (a single Slint callback registration or
// startup step) too internally sequential/closure-heavy to decompose
// further without a compiler in the loop (no `cargo check` is permitted
// for this refactor). Left whole, over the 130-line rule, as the
// documented rare exception it allows for.
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
                let expand_local = if let Some(w) = weak.upgrade() {
                    let st = w.global::<SearchState>();
                    // Always reset the keyboard selection + scroll on (re)open or
                    // refine — never leave a stale "active row" from a prior
                    // search. Arrow nav fires no keystroke, so it is unaffected.
                    st.set_selected_index(-1);
                    st.set_cortinilla_scroll_y(0.0);
                    st.set_cortinilla_open(true);
                    st.set_cortinilla_query(q.clone().into());
                    st.set_cortinilla_loading(true);
                    // Offline OR an unauthenticated (offline) session → the Qobuz
                    // half is empty, so the dropdown is local-only; widen the
                    // on-device section caps.
                    let off = w.global::<OfflineState>();
                    off.get_offline() || off.get_offline_session()
                } else {
                    false
                };
                let cort_version = search::next_cortinilla_version();

                // No cached instant-paint. The cached -> fresh swap (plus the
                // local-fold mid-apply) made the results visibly "jump". Instead
                // the placeholder skeleton (cortinilla-loading) shows while typing
                // and a SINGLE apply paints the real results ~220 ms after the
                // last keystroke — debounced so rapid typing fires one load, not
                // one per keystroke. The version guard drops any stale in-flight
                // load; `load_cortinilla` already folds the on-device section in,
                // so this is one combined paint with no intermediate states.
                {
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    let image_cache = image_cache.clone();
                    let q = q.clone();
                    CORTINILLA_DEBOUNCE.with(|t| {
                        t.start(
                            slint::TimerMode::SingleShot,
                            std::time::Duration::from_millis(220),
                            move || {
                                let runtime = runtime.clone();
                                let weak = weak.clone();
                                let image_cache = image_cache.clone();
                                let q = q.clone();
                                handle.spawn(async move {
                                    match search::load_cortinilla(&runtime, &q, expand_local).await {
                                        Ok((data, local_rows)) => {
                                            let jobs = search::cortinilla_artwork_jobs(&data);
                                            let _ = weak.clone().upgrade_in_event_loop(move |w| {
                                                if search::is_current_cortinilla_version(cort_version) {
                                                    LAST_CORTINILLA.with(|c| {
                                                        *c.borrow_mut() = Some(data.clone())
                                                    });
                                                    LAST_CORTINILLA_LOCAL
                                                        .with(|c| *c.borrow_mut() = local_rows);
                                                    search::apply_cortinilla(&w, data);
                                                }
                                            });
                                            // Mixed payload (Qobuz http / local fs) —
                                            // route each cover by scheme.
                                            artwork::spawn_search_loads(
                                                jobs,
                                                weak.clone(),
                                                image_cache,
                                            );
                                        }
                                        Err(e) => {
                                            log::error!("[qbz-slint] cortinilla load failed: {e}");
                                            let _ = weak.upgrade_in_event_loop(move |w| {
                                                if search::is_current_cortinilla_version(cort_version) {
                                                    w.global::<SearchState>()
                                                        .set_cortinilla_loading(false);
                                                }
                                            });
                                        }
                                    }
                                });
                            },
                        );
                    });
                }
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
