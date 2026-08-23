//! "Add to offline cache" triggers for a whole album or playlist.

use crate::AppWindow;

use super::cache_batch::cache_tracks;
use super::cache_single::Runtime;

/// Cache a whole album for offline playback: fetch its tracks, then batch them.
pub fn cache_album(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    album_id: String,
) {
    let inner = handle.clone();
    handle.spawn(async move {
        let album = match runtime.core().get_album(&album_id).await {
            Ok(a) => a,
            Err(e) => {
                log::error!("[qbz-slint] cache album {album_id} failed: {e}");
                crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't load that album"));
                return;
            }
        };
        let tracks: Vec<qbz_models::Track> = album
            .tracks
            .as_ref()
            .map(|c| c.items.clone())
            .unwrap_or_default();
        if tracks.is_empty() {
            crate::toast::error_weak(&weak, qbz_i18n::t("This album has no playable tracks"));
            return;
        }
        cache_tracks(runtime, weak, inner, tracks);
    });
}

/// Cache a whole playlist for offline playback: fetch its tracks, then batch.
pub fn cache_playlist(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    playlist_id: u64,
) {
    let inner = handle.clone();
    handle.spawn(async move {
        let pl = match runtime.core().get_playlist(playlist_id).await {
            Ok(p) => p,
            Err(e) => {
                log::error!("[qbz-slint] cache playlist {playlist_id} failed: {e}");
                crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't load that playlist"));
                return;
            }
        };
        let tracks: Vec<qbz_models::Track> = pl.tracks.map(|c| c.items).unwrap_or_default();
        if tracks.is_empty() {
            crate::toast::error_weak(&weak, qbz_i18n::t("This playlist has no playable tracks"));
            return;
        }
        cache_tracks(runtime, weak, inner, tracks);
    });
}
