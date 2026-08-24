use crate::*;

pub(crate) fn wire_search(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    wire_search_part1(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_search_part2(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_search_part3(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_search_part4(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_search_part5(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_search_part6(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_search_part7(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_search_part8(window, app_runtime, tokio_rt, image_cache, settings_ctx);
}
