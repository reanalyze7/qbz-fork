use crate::*;

use myqbz::Grid;

/// Mixtapes toolbar: search / sort / view / reset.
pub(crate) fn wire_myqbz_toolbar_mix(
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
}
