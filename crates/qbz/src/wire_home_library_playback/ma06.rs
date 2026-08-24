use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch06(
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
                ("album", "favorite-toggle") => {
                    // The album-header heart: a TRUE toggle that reflects the
                    // favorite-album cache (the card "favorite" arm above is
                    // the same toggle, minus the AlbumState header sync).
                    // Optimistic on the open header, reconciled on the server
                    // result.
                    let Some(w) = weak.upgrade() else {
                        return;
                    };
                    let was_fav = crate::fav_cache::is_album_favorite(&id);
                    let new_state = !was_fav;
                    let st = w.global::<AlbumState>();
                    let is_open = st.get_id() == id.as_str();
                    if is_open {
                        st.set_is_favorite(new_state);
                        st.set_favorite_loading(true);
                    }
                    // Optimistic on every visible album card too (artist
                    // discography, carousels, search, favorites) — reconciled
                    // with the server result below, like the header heart.
                    set_album_row_favorite(&w, &id, new_state);
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let album_id = id.clone();
                    handle.spawn(async move {
                        let res = if new_state {
                            runtime.core().add_favorite("album", &album_id).await
                        } else {
                            runtime.core().remove_favorite("album", &album_id).await
                        };
                        let ok = res.is_ok();
                        if let Err(e) = &res {
                            log::error!(
                                "[qbz-slint] toggle favorite album {album_id} failed: {e}"
                            );
                        }
                        // reco: log the album favorite ADD on success (skip the
                        // un-favorite). Blocking SQLite off the async path.
                        if ok && new_state {
                            let aid = album_id.clone();
                            tokio::task::spawn_blocking(move || {
                                crate::reco::log_favorite_album(aid, None)
                            });
                        }
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            let st = w.global::<AlbumState>();
                            let open_now = st.get_id() == album_id.as_str();
                            if ok {
                                crate::fav_cache::set_album(&album_id, new_state);
                                if open_now {
                                    st.set_favorite_loading(false);
                                    st.set_is_favorite(new_state);
                                }
                                crate::toast::success(
                                    &w,
                                    if new_state {
                                        "Added to favorites"
                                    } else {
                                        "Removed from favorites"
                                    },
                                );
                            } else {
                                if open_now {
                                    st.set_favorite_loading(false);
                                    st.set_is_favorite(was_fav);
                                }
                                // Roll the optimistic card hearts back too.
                                set_album_row_favorite(&w, &album_id, was_fav);
                                crate::toast::error(&w, "Couldn't update favorites");
                            }
                        });
                    });
                }
        _ => {}
    }
}
