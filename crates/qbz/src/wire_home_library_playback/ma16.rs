use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch16(
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
                ("album", "block") | ("album", "unblock") => {
                    if let Some(w) = weak.upgrade() {
                        let st = w.global::<AlbumState>();
                        // Header menu: the open album is AlbumState, so resolve
                        // the display fields (title/artist/cover) from it.
                        let album_id = if id.is_empty() {
                            st.get_id().to_string()
                        } else {
                            id.clone()
                        };
                        if !album_id.is_empty() {
                            let was_blocked =
                                crate::artist_blacklist::is_album_blacklisted(&album_id);
                            // Optimistic flip on the header toggle.
                            st.set_is_album_blocked(!was_blocked);
                            let title = st.get_title().to_string();
                            let artist = st.get_artist().to_string();
                            let cover = st.get_artwork_url().to_string();
                            let res = if was_blocked {
                                crate::artist_blacklist::remove_album(&album_id)
                            } else {
                                crate::artist_blacklist::add_album(
                                    &album_id, &title, &artist, &cover, None,
                                )
                            };
                            match res {
                                Ok(()) => {
                                    seed_blacklist_status(&w);
                                    let msg = if was_blocked {
                                        qbz_i18n::t_args("Album \"{}\" unblocked", &[&title])
                                    } else {
                                        qbz_i18n::t_args("Album \"{}\" blocked", &[&title])
                                    };
                                    crate::toast::success_weak(&weak, msg);
                                }
                                Err(e) => {
                                    log::error!(
                                        "[qbz-slint] album block toggle failed: {e}"
                                    );
                                    st.set_is_album_blocked(was_blocked);
                                    let emsg = if was_blocked {
                                        qbz_i18n::t("Failed to unblock album")
                                    } else {
                                        qbz_i18n::t("Failed to block album")
                                    };
                                    crate::toast::error_weak(&weak, emsg);
                                }
                            }
                        }
                    }
                }
                // Artist card / grid overlay play button: Popular tracks, with
                // a studio-discography fallback when the artist has none (see
                // playback::play_artist).
        _ => {}
    }
}
