use crate::*;

/// Track Info + Album Info modal actions. Split into
/// `wire_track_info_actions` / `wire_album_info_actions` (this dir's
/// `wire_track_info_actions.rs` / `wire_album_info_actions.rs`), called in
/// original order.
pub(crate) fn wire_queue_and_cards_part10(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
    settings_ctx: &Arc<settings::SettingsCtx>,
) {
    let _ = settings_ctx;
    wire_track_info_actions(window, app_runtime, tokio_rt, image_cache);
    wire_album_info_actions(window, app_runtime, tokio_rt, image_cache);
}
