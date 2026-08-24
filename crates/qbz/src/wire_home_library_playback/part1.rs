use crate::*;

// Context-menu / overlay media actions — route play / queue actions into the
// playback controller; favorite / download stay logged.
//
// The original single `on_media_action` closure was ~2000 lines (one giant
// match over every (kind, action) pair emitted by every card/row/menu in the
// app). It is split, in original arm order, into `ma_batch01`..`ma_batch27`
// (ma01.rs..ma27.rs) — each a self-contained match over its own slice of
// arms, called unconditionally in sequence by `dispatch_media_action`
// (ma_dispatch.rs). The local-album play redirect and the Capa-B search
// feedback recording that ran before the big match live in ma_preamble.rs.
pub(crate) fn wire_home_library_playback_part1(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
    settings_ctx: &Arc<settings::SettingsCtx>,
) {
    let _ = settings_ctx;
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.on_media_action(move |kind, id, action| {
            let kind = kind.to_string();
            let id = id.to_string();
            let action = action.to_string();
            log::info!("[qbz-slint] media-action: kind={kind} id={id} action={action}");
            if media_action_local_album_redirect(&weak, &runtime, &handle, &kind, &id, &action) {
                return;
            }
            media_action_record_search_feedback(&weak, &kind, &id, &action);
            dispatch_media_action(&kind, &id, &action, &runtime, &weak, &handle, &image_cache);
        });
    }
}
