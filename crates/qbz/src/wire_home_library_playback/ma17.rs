use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch17(
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
                ("artist", "play") => playback::play_artist(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id.clone(),
                ),
                ("artist", "play-top") => playback::play_artist_top_tracks(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id.clone(),
                ),
                ("artist", "follow") => {
                    // Toggle the artist follow (= Qobuz artist favorite). State
                    // source = the in-memory artist fav cache (seeded by search +
                    // the artist page). Optimistic flip on the cache + every
                    // visible surface (search cards + the ArtistView heart),
                    // revert on network failure.
                    if let (Some(w), Ok(aid)) = (weak.upgrade(), id.parse::<u64>()) {
                        let following = crate::fav_cache::is_artist_favorite(aid);
                        let make = !following;
                        crate::fav_cache::set_artist(aid, make);
                        search::mark_artist_followed(&w, &id, make);
                        let ast = w.global::<ArtistState>();
                        if ast.get_id().as_str() == id.as_str() {
                            ast.set_is_following(make);
                        }
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let artist_id = id.clone();
                        handle.spawn(async move {
                            let res = if make {
                                runtime.core().add_favorite("artist", &artist_id).await
                            } else {
                                runtime.core().remove_favorite("artist", &artist_id).await
                            };
                            match res {
                                Ok(()) => {
                                    // reco: log the favorite only on ADD.
                                    if make {
                                        tokio::task::spawn_blocking(move || {
                                            crate::reco::log_favorite_artist(aid)
                                        });
                                    }
                                }
                                Err(e) => {
                                    log::error!(
                                        "[qbz-slint] toggle follow artist failed: {e}"
                                    );
                                    crate::fav_cache::set_artist(aid, following);
                                    let _ = weak.upgrade_in_event_loop(move |w| {
                                        search::mark_artist_followed(&w, &artist_id, following);
                                        let ast = w.global::<ArtistState>();
                                        if ast.get_id().as_str() == artist_id.as_str() {
                                            ast.set_is_following(following);
                                        }
                                    });
                                }
                            }
                        });
                    }
                }
                // "Not interested" (reco-scoped dismissal — NOT the app-wide
                // blacklist): persist the dismissal, drop the card from the
                // Recommendations rails live, and backfill the freed slot from
                // the retained overflow. The artist stays visible everywhere
                // else (search/home/label pages); future paints exclude it via
                // the §B filter.
        _ => {}
    }
}
