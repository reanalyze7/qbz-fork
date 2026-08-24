use crate::*;

pub(crate) fn wire_home_library_playback(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    wire_home_library_playback_part1(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_home_library_playback_part2(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_home_library_playback_part3(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_home_library_playback_part4(window, app_runtime, tokio_rt, image_cache, settings_ctx);
}
