use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch14(
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
                ("track", "go-to-album") => {
                    // Playlist-detail local sidecar rows first (owner
                    // improvement — Tauri omits the entries there): their
                    // snapshot ids are library row ids, NOT catalog ids, and
                    // the snapshot QueueTrack's album_id already carries the
                    // LOCAL navigation key (the same one the now-playing bar
                    // navigates by — group key). Qobuz + offline-copy rows fall
                    // through to the catalog resolve below (an offline copy's
                    // row id IS its Qobuz id).
                    if let Some(w) = weak.upgrade() {
                        if snapshot_detail_open(&w) {
                            if let Some(qt) = local_playlist::queue_track_for_row(&id) {
                                if matches!(qt.source.as_deref(), Some("local")) {
                                    match qt.album_id.filter(|k| !k.is_empty()) {
                                        Some(key) => w.invoke_open_album(key.into()),
                                        None => log::debug!(
                                            "[qbz-slint] go-to-album: playlist row {id} has no album key"
                                        ),
                                    }
                                    return;
                                }
                            }
                        }
                    }
                    // The menu only carries the track id — resolve the
                    // track to find its album, then open it.
                    if let Ok(track_id) = id.parse::<u64>() {
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        handle.spawn(async move {
                            if let Ok(track) = runtime.core().get_track(track_id).await {
                                if let Some(album_id) =
                                    track.album.as_ref().map(|a| a.id.clone())
                                {
                                    let _ = weak.upgrade_in_event_loop(move |w| {
                                        w.invoke_open_album(album_id.into());
                                    });
                                }
                            }
                        });
                    }
                }
                ("track", "go-to-artist") => {
                    // Same local diversion as go-to-album: local
                    // artists have no id, so route by NAME to the LocalLibrary
                    // Artists tab (the open-artist callback's split).
                    if let Some(w) = weak.upgrade() {
                        if snapshot_detail_open(&w) {
                            if let Some(qt) = local_playlist::queue_track_for_row(&id) {
                                if matches!(qt.source.as_deref(), Some("local")) {
                                    if qt.artist.trim().is_empty() {
                                        log::debug!(
                                            "[qbz-slint] go-to-artist: playlist row {id} has no artist name"
                                        );
                                    } else {
                                        w.invoke_open_artist(qt.artist.into());
                                    }
                                    return;
                                }
                            }
                        }
                    }
                    if let Ok(track_id) = id.parse::<u64>() {
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        handle.spawn(async move {
                            if let Ok(track) = runtime.core().get_track(track_id).await {
                                if let Some(artist_id) =
                                    track.performer.as_ref().map(|p| p.id)
                                {
                                    let _ = weak.upgrade_in_event_loop(move |w| {
                                        w.invoke_open_artist(artist_id.to_string().into());
                                    });
                                }
                            }
                        });
                    }
                }
                // Clickable artist name (album cards) -> artist page.
        _ => {}
    }
}
