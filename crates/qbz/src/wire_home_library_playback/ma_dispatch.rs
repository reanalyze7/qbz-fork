use crate::*;

// Dispatches one `on_media_action(kind, id, action)` event to the batch
// that owns its (kind, action) pattern. Each `ma_batchNN` is a `match`
// over its own slice of the original arms (see ma01.rs..ma27.rs); calling
// every batch unconditionally is safe and preserves original behavior —
// exactly one batch's `match` fires, the rest fall through their own
// `_ => {}`.
pub(crate) fn dispatch_media_action(
    kind: &str,
    id: &str,
    action: &str,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) {
    ma_batch01(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch02(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch03(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch04(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch05(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch06(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch07(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch08(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch09(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch10(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch11(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch12(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch13(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch14(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch15(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch16(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch17(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch18(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch19(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch20(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch21(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch22(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch23(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch24(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch25(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch26(kind, id, action, runtime, weak, handle, image_cache);
    ma_batch27(kind, id, action, runtime, weak, handle, image_cache);
}
