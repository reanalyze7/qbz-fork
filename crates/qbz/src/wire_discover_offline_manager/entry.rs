use crate::*;

pub(crate) fn wire_discover_offline_manager(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    wire_discover_offline_manager_part1(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_discover_offline_manager_part2(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_discover_offline_manager_part3(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_discover_offline_manager_part4(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_discover_offline_manager_part5(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_discover_offline_manager_part6(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_discover_offline_manager_part7(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_discover_offline_manager_part8(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_discover_offline_manager_part9(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_discover_offline_manager_part10(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_discover_offline_manager_part11(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_discover_offline_manager_part12(window, app_runtime, tokio_rt, image_cache, settings_ctx);
}
