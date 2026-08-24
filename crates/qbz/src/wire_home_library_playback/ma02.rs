use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch02(
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
                ("album", "info") => {
                    if !is_local_album_key(&id) {
                        info_modals::open_album_credits(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                        );
                    }
                }
                // Album booklet (digital liner-notes PDF) — the album-header
                // booklet button DOWNLOADS the goody PDF (stashed by
                // album::apply_album) to a user-chosen location. No-op when the
                // album bundles no booklet (empty stashed URL).
                ("album", "booklet") => {
                    crate::booklet::download_booklet(weak.clone(), handle.clone());
                }
                // "From the same artist" carousel "View all" — open the artist's
                // full Albums discography page. `id` is the artist id; reuse the
                // dedicated releases page (release_type "album").
                ("artist", "releases") => {
                    if !id.is_empty() {
                        let name = weak
                            .upgrade()
                            .map(|w| w.global::<AlbumState>().get_artist().to_string())
                            .unwrap_or_default();
                        nav::record(nav::NavEntry::ArtistReleases {
                            id: id.clone(),
                            name: name.clone(),
                            release_type: "album".to_string(),
                        });
                        navigate_artist_releases(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            id.clone(),
                            name,
                            "album".to_string(),
                        );
                        if let Some(w) = weak.upgrade() {
                            update_nav_flags(&w);
                        }
                    }
                }
                ("album", "play") => {
                    // A local id is a metadata group key, not a Qobuz id —
                    // play it from the local cache (Home "Recently played",
                    // etc.) instead of trying to fetch a Qobuz album.
                    if is_local_album_key(&id) {
                        playback::play_local_album(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                            None,
                        );
                    } else {
                        playback::play_album(runtime.clone(), weak.clone(), handle.clone(), id, 0);
                    }
                }
        _ => {}
    }
}
