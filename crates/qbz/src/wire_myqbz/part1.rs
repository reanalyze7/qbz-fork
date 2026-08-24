// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this file holds one
// tightly-sequential Rust function whose internal ordering/control-flow and
// captured-closure state make it unsafe to decompose further without a
// compiler in the loop (no `cargo check` is permitted for this refactor —
// see refactor-plans/crates__qbz__src__main.rs.md). Left whole, over the
// 130-line rule, as the documented rare exception it allows for. (A real
// split, like wire_playlist_manager got, is recommended on a follow-up PR
// with compiler feedback available.)
use crate::*;

/// Wire the My QBZ (Mixtapes & Collections) index grids. READ-ONLY slice:
/// `open-card` / `create-*` are logging STUBS; the toolbar callbacks
/// (search / sort / view / kind-filter / reset) drive `crate::myqbz` rebuilds
/// + re-issue mosaic artwork jobs. Mirrors `wire_playlist_manager`.
pub(crate) fn wire_myqbz(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    use myqbz::Grid;

    // Re-issue mosaic artwork jobs for a grid after a toolbar rebuild (the
    // row set / order changed, so visible cards need their covers reloaded).
    fn refresh_covers(window: &AppWindow, grid: Grid, image_cache: &artwork::ImageCache) {
        let jobs = myqbz::artwork_jobs(window, grid);
        artwork::spawn_loads(jobs, window.as_weak(), image_cache.clone());
    }

    // --- Open a card -> the collection-detail view (Phase-2 Slice 3) -----
    // NAV-IN: record history + navigate (loads via myqbz_detail::navigate),
    // mirroring the grid's own nav and the album/playlist detail openers.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        window.global::<MyQbzActions>().on_open_card(move |id| {
            nav::record(nav::NavEntry::MixtapeDetail(id.to_string()));
            myqbz_detail::navigate(
                runtime.clone(),
                weak.clone(),
                handle.clone(),
                image_cache.clone(),
                id.to_string(),
            );
        });
    }

    // --- Create CTAs: open the create modal pre-set to the right kind ---
    // The kind is fixed by which grid opened it (Mixtapes -> mixtape;
    // Collections -> collection); the modal radio can flip it. Mirrors
    // Tauri's `openCreateModal(kind)`.
    fn open_create_modal(window: &AppWindow, kind: &str) {
        let st = window.global::<MyQbzCreateState>();
        st.set_kind(kind.into());
        st.set_name("".into());
        st.set_creating(false);
        st.set_open(true);
    }
    {
        let weak = window.as_weak();
        window.global::<MyQbzActions>().on_create_mixtape(move || {
            if let Some(w) = weak.upgrade() {
                open_create_modal(&w, "mixtape");
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<MyQbzActions>().on_create_collection(move || {
            if let Some(w) = weak.upgrade() {
                open_create_modal(&w, "collection");
            }
        });
    }

    // --- Create modal: cancel / submit ----------------------------------
    {
        let weak = window.as_weak();
        window.global::<MyQbzCreateActions>().on_close(move || {
            if let Some(w) = weak.upgrade() {
                w.global::<MyQbzCreateState>().set_open(false);
            }
        });
    }
    {
        // Submit: create the collection on a blocking worker, then close the
        // modal + drop the user straight into the new collection's detail
        // view (mirrors Tauri's `submitCreateModal` → `openMixtapeDetail`).
        // The grid is reloaded from the DB on back-nav, so the prepended row
        // shows up there.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        window.global::<MyQbzCreateActions>().on_submit(move || {
            let Some(w) = weak.upgrade() else { return; };
            let st = w.global::<MyQbzCreateState>();
            let name = st.get_name().to_string();
            if name.trim().is_empty() || st.get_creating() {
                return;
            }
            let kind = myqbz::kind_from_str(st.get_kind().as_str());
            st.set_creating(true);

            let weak = weak.clone();
            let handle = handle.clone();
            let image_cache = image_cache.clone();
            let runtime = runtime.clone();
            handle.clone().spawn(async move {
                let nm = name.trim().to_string();
                let created =
                    tokio::task::spawn_blocking(move || myqbz::create_collection(kind, &nm))
                        .await
                        .ok()
                        .flatten();

                let weak2 = weak.clone();
                let handle2 = handle.clone();
                let image_cache2 = image_cache.clone();
                let runtime2 = runtime.clone();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let st = w.global::<MyQbzCreateState>();
                    st.set_creating(false);
                    match created {
                        Some(c) => {
                            st.set_open(false);
                            st.set_name("".into());
                            // Drop into the new collection's detail view.
                            nav::record(nav::NavEntry::MixtapeDetail(c.id.clone()));
                            myqbz_detail::navigate(
                                runtime2.clone(),
                                weak2.clone(),
                                handle2.clone(),
                                image_cache2.clone(),
                                c.id.clone(),
                            );
                        }
                        None => {
                            crate::toast::error(&w, "Failed to create collection");
                        }
                    }
                });
            });
        });
    }

    // --- Add to Mixtape/Collection picker (global singleton) ------------
    {
        // close — clear the pending payload + hide.
        let weak = window.as_weak();
        window.global::<MyQbzAddActions>().on_close(move || {
            if let Some(w) = weak.upgrade() {
                myqbz_add::close(&w);
            }
        });
    }
    {
        // search — re-filter the loaded rows client-side.
        let weak = window.as_weak();
        window
            .global::<MyQbzAddActions>()
            .on_search_changed(move |_query| {
                if let Some(w) = weak.upgrade() {
                    myqbz_add::rebuild(&w);
                }
            });
    }
    {
        // show-create — open the create sub-panel preset to a kind.
        let weak = window.as_weak();
        window
            .global::<MyQbzAddActions>()
            .on_show_create(move |kind| {
                if let Some(w) = weak.upgrade() {
                    let st = w.global::<MyQbzAddState>();
                    st.set_create_kind(kind);
                    st.set_create_name("".into());
                    st.set_creating(true);
                }
            });
    }
    {
        // create-back — return to the picker list.
        let weak = window.as_weak();
        window.global::<MyQbzAddActions>().on_create_back(move || {
            if let Some(w) = weak.upgrade() {
                w.global::<MyQbzAddState>().set_creating(false);
            }
        });
    }
    {
        // pick — add the pending items to the chosen collection.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<MyQbzAddActions>()
            .on_pick(move |collection_id| {
                let Some(w) = weak.upgrade() else { return };
                let st = w.global::<MyQbzAddState>();
                if st.get_busy_id() != "" {
                    return;
                }
                st.set_busy_id(collection_id.clone());
                // The chosen collection's display name (for the toast).
                let name = myqbz_add_row_name(&w, collection_id.as_str());
                let items = myqbz_add::take_pending();
                let cid = collection_id.to_string();

                let weak = weak.clone();
                handle.spawn(async move {
                    let outcome = tokio::task::spawn_blocking(move || {
                        myqbz_add::add_items(&cid, &items)
                    })
                    .await
                    .unwrap_or(myqbz_add::AddOutcome { added: 0, skipped: 0 });
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        myqbz_add::toast_outcome(&w, &name, &outcome);
                        myqbz_add::close(&w);
                    });
                });
            });
    }
    {
        // create-and-add — create a new collection then add the items.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<MyQbzAddActions>().on_create_and_add(move || {
            let Some(w) = weak.upgrade() else { return };
            let st = w.global::<MyQbzAddState>();
            let name = st.get_create_name().trim().to_string();
            if name.is_empty() || st.get_create_busy() {
                return;
            }
            let kind = st.get_create_kind().to_string();
            st.set_create_busy(true);
            let items = myqbz_add::take_pending();

            let weak = weak.clone();
            handle.spawn(async move {
                let created = {
                    let kind = kind.clone();
                    let name = name.clone();
                    tokio::task::spawn_blocking(move || {
                        myqbz_add::create_collection(&kind, &name)
                    })
                    .await
                    .ok()
                    .flatten()
                };
                match created {
                    Some((cid, cname)) => {
                        let outcome = tokio::task::spawn_blocking(move || {
                            myqbz_add::add_items(&cid, &items)
                        })
                        .await
                        .unwrap_or(myqbz_add::AddOutcome { added: 0, skipped: 0 });
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            myqbz_add::toast_outcome(&w, &cname, &outcome);
                            myqbz_add::close(&w);
                        });
                    }
                    None => {
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            w.global::<MyQbzAddState>().set_create_busy(false);
                            crate::toast::error(&w, "Failed to create");
                        });
                    }
                }
            });
        });
    }

    // --- Mixtapes toolbar -----------------------------------------------
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<MyQbzActions>()
            .on_mix_search_changed(move |query| {
                if let Some(w) = weak.upgrade() {
                    w.global::<MyQbzState>().set_mix_search(query);
                    myqbz::rebuild(&w, Grid::Mixtapes);
                    refresh_covers(&w, Grid::Mixtapes, &image_cache);
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window.global::<MyQbzActions>().on_mix_set_sort(move |field| {
            if let Some(w) = weak.upgrade() {
                myqbz::set_sort(&w, Grid::Mixtapes, field.as_str());
                refresh_covers(&w, Grid::Mixtapes, &image_cache);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<MyQbzActions>().on_mix_set_view(move |view| {
            if let Some(w) = weak.upgrade() {
                w.global::<MyQbzState>().set_mix_view(view);
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window.global::<MyQbzActions>().on_mix_reset(move || {
            if let Some(w) = weak.upgrade() {
                myqbz::reset(&w, Grid::Mixtapes);
                refresh_covers(&w, Grid::Mixtapes, &image_cache);
            }
        });
    }

    // --- Collections toolbar --------------------------------------------
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<MyQbzActions>()
            .on_col_search_changed(move |query| {
                if let Some(w) = weak.upgrade() {
                    w.global::<MyQbzState>().set_col_search(query);
                    myqbz::rebuild(&w, Grid::Collections);
                    refresh_covers(&w, Grid::Collections, &image_cache);
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window.global::<MyQbzActions>().on_col_set_sort(move |field| {
            if let Some(w) = weak.upgrade() {
                myqbz::set_sort(&w, Grid::Collections, field.as_str());
                refresh_covers(&w, Grid::Collections, &image_cache);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<MyQbzActions>().on_col_set_view(move |view| {
            if let Some(w) = weak.upgrade() {
                w.global::<MyQbzState>().set_col_view(view);
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<MyQbzActions>()
            .on_col_set_kind_filter(move |kind| {
                if let Some(w) = weak.upgrade() {
                    w.global::<MyQbzState>().set_col_kind_filter(kind);
                    myqbz::rebuild(&w, Grid::Collections);
                    refresh_covers(&w, Grid::Collections, &image_cache);
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window.global::<MyQbzActions>().on_col_reset(move || {
            if let Some(w) = weak.upgrade() {
                myqbz::reset(&w, Grid::Collections);
                refresh_covers(&w, Grid::Collections, &image_cache);
            }
        });
    }
}

