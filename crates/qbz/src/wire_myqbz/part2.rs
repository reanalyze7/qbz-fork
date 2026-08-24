use crate::*;

/// Wire the My QBZ collection-DETAIL view (Phase-2 Slice 3, read-only). Split
/// into `wire_myqbz_detail_*` sub-functions (this dir's `detail_*.rs`), each
/// registering a contiguous run of the original callbacks in their original
/// relative order. Every hero CTA + per-row context action beyond what's
/// implemented stays a logging STUB — the read-only boundary for this slice.
pub(crate) fn wire_myqbz_detail(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    // Stash the runtime for the mutation-reload paths (cover/edit) that
    // re-run `myqbz_detail::navigate` (whose resolveItems pass needs it)
    // without threading it through every entry point.
    myqbz_detail::set_runtime(app_runtime.clone());
    // Blacklist Manager album-cover resolution needs the shared image cache.
    blacklist_manager::set_image_cache(image_cache.clone());

    wire_myqbz_detail_toolbar(window, app_runtime, tokio_rt, image_cache);
    wire_myqbz_detail_open(window, app_runtime, tokio_rt, image_cache);
    wire_myqbz_detail_hero(window, app_runtime, tokio_rt, image_cache);
    wire_myqbz_detail_bulk(window, app_runtime, tokio_rt, image_cache);
    wire_myqbz_detail_overflow(window, app_runtime, tokio_rt, image_cache);
    wire_myqbz_detail_edit(window, app_runtime, tokio_rt, image_cache);
    wire_myqbz_detail_rows(window, app_runtime, tokio_rt, image_cache);
    wire_myqbz_detail_expanded(window, app_runtime, tokio_rt, image_cache);
}
