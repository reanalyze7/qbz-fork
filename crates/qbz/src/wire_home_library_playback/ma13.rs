use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch13(
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
                ("track", "share-songlink") => {
                    // Tauri-parity resolution (#514): fetch the track to get
                    // its ISRC, then ISRC -> Deezer -> song.link. The old
                    // URL-only Odesli call never worked for Qobuz input
                    // (could_not_resolve_entity) — see share.rs.
                    let track = id.clone();
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    crate::toast::info_weak(&weak, qbz_i18n::t("Fetching Song.link..."));
                    handle.spawn(async move {
                        let isrc = match track.parse::<u64>() {
                            Ok(tid) => runtime
                                .core()
                                .get_track(tid)
                                .await
                                .ok()
                                .and_then(|t| t.isrc),
                            Err(_) => None,
                        };
                        match share::songlink_for_track(&track, isrc.as_deref()).await {
                            Some(url) => {
                                share::copy_to_clipboard(url);
                                log::info!("[qbz-slint] copied Song.link for track {track}");
                                crate::toast::success_weak(&weak, qbz_i18n::t("Link copied"));
                            }
                            None => {
                                log::warn!("[qbz-slint] Song.link resolution failed for {track}");
                                crate::toast::error_weak(
                                    &weak,
                                    qbz_i18n::t("Failed to copy link"),
                                );
                            }
                        }
                    });
                }
        _ => {}
    }
}
