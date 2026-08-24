use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch08(
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
    let _image_cache = image_cache.clone();
    let id = id.to_string();
    match (kind, action) {
                ("album", "share-qobuz") => {
                    share::copy_to_clipboard(share::qobuz_album_url(&id));
                    log::info!("[qbz-slint] copied Qobuz link for album {id}");
                }
                ("album", "share-songlink") => {
                    // Tauri-parity resolution (#514): fetch the album to get
                    // its UPC, then UPC -> Deezer -> album.link. The old
                    // URL-only Odesli call never worked for Qobuz input
                    // (could_not_resolve_entity) — see share.rs.
                    let album = id.clone();
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    crate::toast::info_weak(&weak, qbz_i18n::t("Fetching Album.link..."));
                    handle.spawn(async move {
                        let upc = runtime
                            .core()
                            .get_album(&album)
                            .await
                            .ok()
                            .and_then(|a| a.upc);
                        match share::albumlink_for_album(&album, upc.as_deref()).await {
                            Some(url) => {
                                share::copy_to_clipboard(url);
                                log::info!("[qbz-slint] copied Album.link for album {album}");
                                crate::toast::success_weak(&weak, qbz_i18n::t("Link copied"));
                            }
                            None => {
                                log::warn!("[qbz-slint] Album.link resolution failed for {album}");
                                crate::toast::error_weak(
                                    &weak,
                                    qbz_i18n::t("Failed to copy link"),
                                );
                            }
                        }
                    });
                }
                ("track", "play-next") => {
                    // Source-typed routing — see the ("track","queue") arm
                    // (same seam, insert-next instead of append).
                    if let Some(w) = weak.upgrade() {
                        if snapshot_detail_open(&w) {
                            if let Some(qt) = local_playlist::queue_track_for_row(&id) {
                                if matches!(qt.source.as_deref(), Some("local")) {
                                    playback::enqueue_queue_tracks(
                                        runtime.clone(),
                                        weak.clone(),
                                        handle.clone(),
                                        vec![qt],
                                        true,
                                    );
                                    return;
                                }
                            }
                        }
                    }
                    if let Ok(track_id) = id.parse::<u64>() {
                        playback::play_track_next(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
        _ => {}
    }
}
