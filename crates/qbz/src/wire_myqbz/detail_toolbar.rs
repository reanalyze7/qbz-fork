use crate::*;

use MyQbzDetailActions as Act;

/// Toolbar (client-side re-derive): search / sort / type-filter /
/// source-filter / view-mode / select-mode / reset.
pub(crate) fn wire_myqbz_detail_toolbar(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_search_changed(move |q| {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::search(&w, q.as_str());
                refresh_row_covers(&w, &image_cache);
                ensure_expanded_if_active(&w, &runtime, &handle);
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_set_sort(move |field| {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::set_sort(&w, field.as_str());
                refresh_row_covers(&w, &image_cache);
                ensure_expanded_if_active(&w, &runtime, &handle);
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_set_type_filter(move |value| {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::set_type_filter(&w, value.as_str());
                refresh_row_covers(&w, &image_cache);
                ensure_expanded_if_active(&w, &runtime, &handle);
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_toggle_source_filter(move |kind| {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::toggle_source_filter(&w, kind.as_str());
                refresh_row_covers(&w, &image_cache);
                ensure_expanded_if_active(&w, &runtime, &handle);
            }
        });
    }
    {
        let weak = window.as_weak();
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_set_view_mode(move |mode| {
            if let Some(w) = weak.upgrade() {
                // Sets view-mode + persists the per-collection prefs (spec §18).
                myqbz_detail::set_view_mode(&w, mode.as_str());
                // Entering expanded mode: fetch every expandable item's tracks
                // (spec §8 — tracks render directly under each row).
                if mode == "expanded" {
                    myqbz_detail::ensure_expanded(runtime.clone(), weak.clone(), handle.clone());
                }
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<Act>().on_toggle_select_mode(move || {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::toggle_select_mode(&w);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<Act>().on_toggle_item_select(move |position| {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::toggle_item_select(&w, position);
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window.global::<Act>().on_reset_filters(move || {
            if let Some(w) = weak.upgrade() {
                myqbz_detail::reset_filters(&w);
                refresh_row_covers(&w, &image_cache);
                ensure_expanded_if_active(&w, &runtime, &handle);
            }
        });
    }
}
