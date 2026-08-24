use crate::*;

// Cortinilla (live dropdown) live-search branch, only when the search
// module is ON (D5). Split out of the single `on_live` callback
// (wire_search_part1, part1.rs) to stay under the 130-line file cap — pure
// extraction, called inline in place of the original `if
// crate::search_service::is_enabled() { ... }` block.
pub(crate) fn spawn_cortinilla_live_search(
    weak: &slint::Weak<AppWindow>,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
    q: &str,
) {
    let weak = weak.clone();
    let runtime = runtime.clone();
    let handle = handle.clone();
    let image_cache = image_cache.clone();
    let q = q.to_string();
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
