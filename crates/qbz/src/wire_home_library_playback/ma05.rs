use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch05(
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
                ("album", "favorite") => {
                    // Album-card heart + "…" menu entry: a TRUE TOGGLE keyed
                    // off the favorite-album cache (filled heart → remove,
                    // empty → add), mirroring the header "favorite-toggle"
                    // arm below. Was add-only while the cards couldn't show
                    // favorite state; now that they do, re-adding from a
                    // filled heart would lie. Optimistic: flip the heart on
                    // every visible card right away (mirrors the track
                    // rows); rolled back on failure. NOTE: the Favorites
                    // albums tab never reaches this arm — FavoritesView
                    // intercepts "favorite" to unfavorite-album (fade-out +
                    // row removal).
                    let was_fav = crate::fav_cache::is_album_favorite(&id);
                    let new_state = !was_fav;
                    if let Some(w) = weak.upgrade() {
                        set_album_row_favorite(&w, &id, new_state);
                    }
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let album_id = id.clone();
                    handle.spawn(async move {
                        let res = if new_state {
                            runtime.core().add_favorite("album", &album_id).await
                        } else {
                            runtime.core().remove_favorite("album", &album_id).await
                        };
                        match res {
                            Ok(()) => {
                                // Keep the favorite-album cache in sync so the
                                // album-header heart reflects a card toggle.
                                crate::fav_cache::set_album(&album_id, new_state);
                                crate::toast::success_weak(
                                    &weak,
                                    if new_state {
                                        "Added to favorites"
                                    } else {
                                        "Removed from favorites"
                                    },
                                );
                                // reco: log the album favorite ADD on success
                                // only — Capa B scores adds, never removals.
                                if new_state {
                                    let aid = album_id.clone();
                                    tokio::task::spawn_blocking(move || {
                                        crate::reco::log_favorite_album(aid, None)
                                    });
                                }
                            }
                            Err(e) => {
                                log::error!(
                                    "[qbz-slint] toggle favorite album {album_id} failed: {e}"
                                );
                                crate::toast::error_weak(&weak, "Couldn't update favorites");
                                // Roll the optimistic hearts back to the
                                // pre-click state.
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    set_album_row_favorite(&w, &album_id, was_fav);
                                });
                            }
                        }
                    });
                }
        _ => {}
    }
}
