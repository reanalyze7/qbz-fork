//! Artist-card play button: Popular tracks, falling back to the studio
//! discography when the artist has no top tracks.

use super::artist_fetch::make_top_track_queue;
use super::artist_studio::studio_discography_queue;
use super::super::engine::after_track_change;
use super::super::queue_context::stamp_queue_context;
use super::super::recent_blacklist::filter_blacklisted_queue;
use super::super::state::refresh_sidebar;
use super::super::Runtime;
use crate::AppWindow;
use qbz_models::QueueTrack;

/// Play button on an artist card / grid overlay. Plays the artist's Popular
/// (top) tracks; if the artist has NONE, falls back to their STUDIO
/// discography — the "album" + EP/single buckets, in the page's section order,
/// deduped by album id — EXCLUDING compilations, live, and "other". One fresh
/// queue starting at the first track. Wired to media-action("artist", id,
/// "play"). Fetches the artist page ONCE and decides from it (no double
/// round-trip for the top-tracks-present common case).
pub fn play_artist(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    artist_id: String,
) {
    handle.spawn(async move {
        let id: u64 = match artist_id.parse() {
            Ok(v) => v,
            Err(_) => {
                log::warn!("[qbz-slint] artist-play: invalid artist id {artist_id}");
                return;
            }
        };
        let page = match runtime.core().get_artist_page(id, None).await {
            Ok(p) => p,
            Err(e) => {
                log::error!("[qbz-slint] artist-play: get_artist_page {artist_id} failed: {e}");
                crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't load this artist"));
                return;
            }
        };
        let artist_name = page.name.display.clone();

        // 1) Popular tracks — the primary behavior. Moves `top_tracks` out of
        // `page`; `releases` (a disjoint field) is moved later in the fallback.
        let raw_top: Vec<QueueTrack> = page
            .top_tracks
            .unwrap_or_default()
            .into_iter()
            .map(|track| make_top_track_queue(track, &artist_name))
            .collect();
        let top = filter_blacklisted_queue(raw_top);
        if !top.is_empty() {
            let mut tracks = top;
            stamp_queue_context(&mut tracks, "artist", &artist_id);
            let start_track_id = tracks[0].id;
            runtime.core().set_queue(tracks, Some(0)).await;
            after_track_change(&runtime, &weak, start_track_id).await;
            refresh_sidebar(true);
            return;
        }

        // 2) Fallback — the studio discography (see `artist_studio.rs`).
        let Some(mut queue) =
            studio_discography_queue(&runtime, &artist_id, page.releases.unwrap_or_default()).await
        else {
            crate::toast::error_weak(&weak, qbz_i18n::t("No top tracks available for this artist"));
            return;
        };
        stamp_queue_context(&mut queue, "artist", &artist_id);
        let start_track_id = queue[0].id;
        runtime.core().set_queue(queue, Some(0)).await;
        after_track_change(&runtime, &weak, start_track_id).await;
        refresh_sidebar(true);
    });
}
