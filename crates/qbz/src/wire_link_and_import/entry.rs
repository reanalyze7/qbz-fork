use crate::*;

pub(crate) fn wire_link_and_import(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    wire_link_and_import_part1(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_link_and_import_part2(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_link_and_import_part3(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_link_and_import_part4(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_link_and_import_part5(window, app_runtime, tokio_rt, image_cache, settings_ctx);
}
