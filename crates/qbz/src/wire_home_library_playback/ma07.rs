use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch07(
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
                ("album", "cache") => offline_cache::cache_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                ),
                ("album", "recache") => offline_cache::redownload_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                    // Refresh the WHOLE album (Tauri's "Refresh offline copy"
                    // re-downloads every track, not only the failed ones).
                    false,
                ),
                ("album", "add-to-playlist") => {
                    // Resolve the album's loaded tracks to their Qobuz catalog
                    // ids and open the playlist picker for the whole set
                    // (mirrors Tauri's album → Add to playlist). Local
                    // albums carry no catalog ids, so the entry no-ops there
                    // (the header menu is a Qobuz surface).
                    let Some(w) = weak.upgrade() else {
                        return;
                    };
                    let ids: Vec<String> = {
                        use slint::Model;
                        w.global::<AlbumState>()
                            .get_tracks()
                            .iter()
                            .map(|t| t.id.to_string())
                            .filter(|s| s.parse::<u64>().is_ok())
                            .collect()
                    };
                    if ids.is_empty() {
                        toast::error(&w, "No tracks to add");
                        return;
                    }
                    playlist_picker::open_multi(&w, &ids, false);
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    handle.spawn(async move {
                        let playlists = playlist_picker::load(&runtime).await;
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            playlist_picker::apply(&w, playlists);
                        });
                    });
                }
        _ => {}
    }
}
