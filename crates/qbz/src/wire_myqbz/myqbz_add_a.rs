use crate::*;

/// Add to Mixtape/Collection picker (global singleton): close, search,
/// show-create, and create-back.
pub(crate) fn wire_myqbz_add_a(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    let _ = app_runtime;
    let _ = tokio_rt;
    let _ = image_cache;
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
}
