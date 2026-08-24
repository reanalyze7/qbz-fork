use crate::*;

/// Offline Cache Manager actions. Split into `wire_offline_manager_a` /
/// `wire_offline_manager_b` (this dir's `wire_offline_manager_a.rs` /
/// `wire_offline_manager_b.rs`), called in original order.
pub(crate) fn wire_discover_offline_manager_part3(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
    settings_ctx: &Arc<settings::SettingsCtx>,
) {
    let _ = image_cache;
    let _ = settings_ctx;
    let runtime = app_runtime.clone();
    let handle = tokio_rt.handle().clone();
    wire_offline_manager_a(window, &runtime, &handle);
    wire_offline_manager_b(window, &runtime, &handle);
}
