use crate::*;

/// Local Library per-row track actions (context menu / row affordances).
/// Split into `local_track_action_a`/`_b`/`_c` (this dir's
/// `local_track_action_*.rs`), called unconditionally in sequence from the
/// single `on_track_action` registration — safe since each action matches
/// at most one of the three, the others fall through their own `_ => {}`
/// (the unhandled-action log fallback lives in `_c`, the last one called).
pub(crate) fn wire_local_library_settings_part8(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
    settings_ctx: &Arc<settings::SettingsCtx>,
) {
    let _ = image_cache;
    let _ = settings_ctx;
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_track_action(move |id, action| {
                local_track_action_a(id.as_str(), action.as_str(), &runtime, &weak, &handle);
                local_track_action_b(id.as_str(), action.as_str(), &runtime, &weak, &handle);
                local_track_action_c(id.as_str(), action.as_str(), &runtime, &weak, &handle);
            });
    }
}
