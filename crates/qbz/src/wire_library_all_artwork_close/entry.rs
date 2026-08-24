use crate::*;

pub(crate) fn wire_library_all_artwork_close(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    wire_library_all_artwork_close_part1(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_library_all_artwork_close_part2(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_library_all_artwork_close_part3(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_library_all_artwork_close_part4(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_library_all_artwork_close_part5(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_library_all_artwork_close_part6(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_library_all_artwork_close_part7(window, app_runtime, tokio_rt, image_cache, settings_ctx);
}
