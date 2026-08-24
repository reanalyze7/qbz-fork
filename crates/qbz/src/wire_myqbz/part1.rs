use crate::*;

/// Wire the My QBZ (Mixtapes & Collections) index grids. READ-ONLY slice:
/// `open-card` / `create-*` are logging STUBS; the toolbar callbacks
/// (search / sort / view / kind-filter / reset) drive `crate::myqbz` rebuilds
/// + re-issue mosaic artwork jobs. Mirrors `wire_playlist_manager`. Split
/// into `wire_myqbz_*` sub-functions (this dir's `myqbz_*.rs`), each
/// registering a contiguous run of the original callbacks in original order.
pub(crate) fn wire_myqbz(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    wire_myqbz_open(window, app_runtime, tokio_rt, image_cache);
    wire_myqbz_create(window, app_runtime, tokio_rt, image_cache);
    wire_myqbz_add_a(window, app_runtime, tokio_rt, image_cache);
    wire_myqbz_add_b(window, app_runtime, tokio_rt, image_cache);
    wire_myqbz_toolbar_mix(window, app_runtime, tokio_rt, image_cache);
    wire_myqbz_toolbar_col(window, app_runtime, tokio_rt, image_cache);
}
