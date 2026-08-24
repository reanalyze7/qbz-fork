use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch12(
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
                ("track", "add-to-mixtape") => {
                    // The menu only carries the track id; resolve the Qobuz
                    // track (this entry is gated to Qobuz/offline in the menu)
                    // to build the AddToMixtape payload, then open the picker.
                    if let Ok(track_id) = id.parse::<u64>() {
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let handle2 = handle.clone();
                        handle.spawn(async move {
                            let item = match runtime.core().get_track(track_id).await {
                                Ok(track) => {
                                    let artist = track
                                        .performer
                                        .as_ref()
                                        .map(|p| p.name.clone())
                                        .unwrap_or_default();
                                    let album = track
                                        .album
                                        .as_ref()
                                        .map(|a| a.title.clone())
                                        .unwrap_or_default();
                                    let subtitle = [artist, album]
                                        .into_iter()
                                        .filter(|s| !s.is_empty())
                                        .collect::<Vec<_>>()
                                        .join(" · ");
                                    let artwork_url = track.album.as_ref().and_then(|a| {
                                        a.image
                                            .thumbnail
                                            .clone()
                                            .or_else(|| a.image.small.clone())
                                    });
                                    myqbz_add::AddItem {
                                        item_type: "track".into(),
                                        source: "qobuz".into(),
                                        source_item_id: track_id.to_string(),
                                        title: track.title.clone(),
                                        subtitle: (!subtitle.is_empty()).then_some(subtitle),
                                        artwork_url,
                                        year: None,
                                        track_count: None,
                                    }
                                }
                                Err(e) => {
                                    log::warn!(
                                        "[qbz-slint] add-to-mixtape: get_track {track_id} failed: {e}"
                                    );
                                    return;
                                }
                            };
                            open_add_to_mixtape(weak, handle2, vec![item]);
                        });
                    }
                }
                ("track", "share-qobuz") => {
                    share::copy_to_clipboard(share::qobuz_track_url(&id));
                    log::info!("[qbz-slint] copied Qobuz link for track {id}");
                }
        _ => {}
    }
}
