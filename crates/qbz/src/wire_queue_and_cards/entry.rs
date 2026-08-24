use crate::*;

pub(crate) fn wire_queue_and_cards(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    wire_queue_and_cards_part1(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_queue_and_cards_part2(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_queue_and_cards_part3(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_queue_and_cards_part4(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_queue_and_cards_part5(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_queue_and_cards_part6(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_queue_and_cards_part7(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_queue_and_cards_part8(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_queue_and_cards_part9(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_queue_and_cards_part10(window, app_runtime, tokio_rt, image_cache, settings_ctx);
}
