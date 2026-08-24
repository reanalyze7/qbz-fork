use crate::*;

/// Create CTAs (open the create modal pre-set to the right kind — the kind
/// is fixed by which grid opened it, Mixtapes -> mixtape / Collections ->
/// collection; the modal radio can flip it — mirrors Tauri's
/// `openCreateModal(kind)`) and the create modal's own cancel / submit.
pub(crate) fn wire_myqbz_create(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
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
}
