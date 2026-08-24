use crate::*;

/// Add to Mixtape/Collection picker (global singleton): pick (add the
/// pending items to the chosen collection) and create-and-add (create a new
/// collection then add the items).
pub(crate) fn wire_myqbz_add_b(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    let _ = app_runtime;
    let _ = image_cache;
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
}
