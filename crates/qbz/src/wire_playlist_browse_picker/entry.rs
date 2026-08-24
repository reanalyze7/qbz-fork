use crate::*;

pub(crate) fn wire_playlist_browse_picker(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    wire_playlist_browse_picker_part1(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_browse_picker_part2(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_browse_picker_part3(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_browse_picker_part4(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_browse_picker_part5(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_browse_picker_part6(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_browse_picker_part7(window, app_runtime, tokio_rt, image_cache, settings_ctx);
}
