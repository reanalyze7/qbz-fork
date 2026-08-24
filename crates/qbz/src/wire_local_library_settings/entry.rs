use crate::*;

pub(crate) fn wire_local_library_settings(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    wire_local_library_settings_part1(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_local_library_settings_part2(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_local_library_settings_part3(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_local_library_settings_part4(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_local_library_settings_part5(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_local_library_settings_part6(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_local_library_settings_part7(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_local_library_settings_part8(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_local_library_settings_part9(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_local_library_settings_part10(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_local_library_settings_part11(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_local_library_settings_part12(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_local_library_settings_part13(window, app_runtime, tokio_rt, image_cache, settings_ctx);
}
