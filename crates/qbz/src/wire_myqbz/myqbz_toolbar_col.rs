use crate::*;

use myqbz::Grid;

/// Collections toolbar: search / sort / view / kind-filter / reset.
pub(crate) fn wire_myqbz_toolbar_col(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    let _ = app_runtime;
    let _ = tokio_rt;
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
