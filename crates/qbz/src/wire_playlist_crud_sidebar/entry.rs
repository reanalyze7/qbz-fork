use crate::*;

pub(crate) fn wire_playlist_crud_sidebar(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    wire_playlist_crud_sidebar_part1(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_crud_sidebar_part2(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_crud_sidebar_part3(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_crud_sidebar_part4(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_crud_sidebar_part5(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_crud_sidebar_part6(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_crud_sidebar_part7(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_crud_sidebar_part8(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_crud_sidebar_part9(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_playlist_crud_sidebar_part10(window, app_runtime, tokio_rt, image_cache, settings_ctx);
}
