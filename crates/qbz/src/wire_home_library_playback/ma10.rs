use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch10(
    kind: &str,
    id: &str,
    action: &str,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) {
    let runtime = runtime.clone();
    let weak = weak.clone();
    let handle = handle.clone();
    let image_cache = image_cache.clone();
    let id = id.to_string();
    match (kind, action) {
                ("ext-reco-list", "queue") => {
                    if let Some(w) = weak.upgrade() {
                        let ids = external_reco::list_track_ids(&w, &id);
                        playback::enqueue_track_ids(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            ids,
                            false,
                        );
                    }
                }
                ("ext-reco-list", "create-playlist") => {
                    if let Some(w) = weak.upgrade() {
                        let ids = external_reco::list_track_ids(&w, &id);
                        if !ids.is_empty() {
                            let ids_str: Vec<String> =
                                ids.iter().map(|i| i.to_string()).collect();
                            playlist_picker::open_for_ids(
                                &w,
                                runtime.clone(),
                                &handle,
                                ids_str,
                                false,
                            );
                        }
                    }
                }
        _ => {}
    }
}
