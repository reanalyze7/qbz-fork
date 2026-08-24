use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch15(
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
                ("artist", "open") => {
                    if let Some(w) = weak.upgrade() {
                        w.invoke_open_artist(id.clone().into());
                    }
                }
                // Clickable album name (track rows) -> album page.
                ("album", "open") => {
                    if let Some(w) = weak.upgrade() {
                        w.invoke_open_album(id.clone().into());
                    }
                }
                // Now-playing context (song-card layers button) -> playlist page.
                ("playlist", "open") => {
                    nav::record(nav::NavEntry::Playlist(id.clone()));
                    navigate_playlist(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        id.clone(),
                    );
                }
                // Blacklist / Show toggle from the ArtistView overflow
                // menu (and the hidden-artist banner). Resolves the id
                // from the passed value, falling back to ArtistState.id
                // Reads the name from
                // ArtistState for storage. Optimistic with rollback: flip
                // ArtistState.is-blacklisted immediately, perform the
                // mutation, revert + error-toast on failure. Synchronous
                // on the event-loop thread, so there is no re-entrancy
                // window (a second click can't interleave mid-toggle).
                ("artist", "share") => {
                    let artist_id = if id.is_empty() {
                        weak.upgrade()
                            .map(|w| w.global::<ArtistState>().get_id().to_string())
                            .unwrap_or_default()
                    } else {
                        id.clone()
                    };
                    if !artist_id.is_empty() {
                        share::copy_to_clipboard(share::qobuz_artist_url(&artist_id));
                        if let Some(w) = weak.upgrade() {
                            crate::toast::success(&w, qbz_i18n::t("Link copied"));
                        }
                    }
                }
                ("artist", "blacklist-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let st = w.global::<ArtistState>();
                        let artist_id = if id.is_empty() {
                            st.get_id().to_string()
                        } else {
                            id.clone()
                        };
                        let name = st.get_name().to_string();
                        if let Ok(id_num) = artist_id.parse::<u64>() {
                            let was_blacklisted =
                                crate::artist_blacklist::is_blacklisted(id_num);
                            // Optimistic flip.
                            st.set_is_blacklisted(!was_blacklisted);
                            let res = if was_blacklisted {
                                crate::artist_blacklist::remove(id_num)
                            } else {
                                crate::artist_blacklist::add(
                                    id_num,
                                    &name,
                                    None,
                                )
                            };
                            match res {
                                Ok(()) => {
                                    // Live refresh for the artist page is the
                                    // optimistic ArtistState.is-blacklisted
                                    // flip above (drives the banner + the
                                    // menu Show/Blacklist label). ArtistView
                                    // popular-tracks rows are deliberately
                                    // NOT per-row greyed (T6 scoping — the
                                    // banner is the artist-page surface);
                                    // other open views (search, album,
                                    // favorites) re-stamp on next navigation
                                    // (no global observer).
                                    let msg = if was_blacklisted {
                                        format!("{name} is now visible")
                                    } else {
                                        format!("{name} is now hidden")
                                    };
                                    crate::toast::success_weak(&weak, msg);
                                }
                                Err(e) => {
                                    log::error!(
                                        "[qbz-slint] blacklist toggle failed: {e}"
                                    );
                                    // Rollback the optimistic flip.
                                    st.set_is_blacklisted(was_blacklisted);
                                    crate::toast::error_weak(
                                        &weak,
                                        "Failed to update artist visibility",
                                    );
                                }
                            }
                        }
                    }
                }
        _ => {}
    }
}
