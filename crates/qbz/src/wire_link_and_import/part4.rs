use crate::*;

/// Appearance settings persistence. Split into `wire_appearance_*`
/// sub-functions (this dir's `wire_appearance_*.rs` / `appearance_select_*.rs`),
/// each registering one (or, for the oversized `on_appearance_select` match,
/// delegating to a helper for) callback on the shared `AppearanceState`
/// global, called in original order.
pub(crate) fn wire_link_and_import_part4(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
    settings_ctx: &Arc<settings::SettingsCtx>,
) {
    let _ = app_runtime;
    let _ = tokio_rt;
    let _ = image_cache;
    let _ = settings_ctx;
    wire_appearance_bool(window);
    wire_appearance_select(window);
    wire_appearance_action_custom(window);
}
