use crate::*;

pub(crate) fn wire_create_playlist_dac_import(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    wire_create_playlist_dac_import_part1(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_create_playlist_dac_import_part2(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_create_playlist_dac_import_part3(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_create_playlist_dac_import_part4(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_create_playlist_dac_import_part5(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_create_playlist_dac_import_part6(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_create_playlist_dac_import_part7(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_create_playlist_dac_import_part8(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_create_playlist_dac_import_part9(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_create_playlist_dac_import_part10(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_create_playlist_dac_import_part11(window, app_runtime, tokio_rt, image_cache, settings_ctx);
}
