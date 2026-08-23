//! The studio-discography fallback for `play_artist`, split out to keep
//! both files under the line budget.

use super::super::queue_context::make_queue_track;
use super::super::recent_blacklist::track_is_blacklisted_full;
use super::super::Runtime;
use qbz_models::QueueTrack;

/// Studio discography release_type buckets (webplayer keys). "album" = the
/// discography; epSingle/ep/single = EPs & Singles. compilation/live/other
/// are omitted on purpose (owner spec: studio releases only).
const STUDIO_TYPES: &[&str] = &["album", "epSingle", "ep", "single"];

/// Build a queue from the artist's STUDIO discography (the `play_artist`
/// fallback when the artist has no Popular tracks): the "album" + EP/single
/// buckets, in the page's section order, deduped by album id. Fetches each
/// album quietly — a single unavailable album must not toast (this is a
/// bulk play) and must not abort the rest, just skip it. Returns `None`
/// (after logging) when there are no studio releases, or none produce a
/// playable track.
pub(super) async fn studio_discography_queue(
    runtime: &Runtime,
    artist_id: &str,
    releases: Vec<qbz_models::PageArtistReleaseGroup>,
) -> Option<Vec<QueueTrack>> {
    // Collect album ids from the studio buckets in the page's section order,
    // deduped (a release can appear in more than one bucket for edge metadata).
    let mut album_ids: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for group in releases {
        if STUDIO_TYPES.contains(&group.release_type.as_str()) {
            for item in group.items {
                if seen.insert(item.id.clone()) {
                    album_ids.push(item.id);
                }
            }
        }
    }
    if album_ids.is_empty() {
        log::warn!("[qbz-slint] artist-play: {artist_id} has no top tracks and no studio releases");
        return None;
    }

    // Concatenate each studio album's tracks into one queue. Blacklist
    // filtering mirrors `fetch_album_for_play` (performer/composer/featured
    // aware).
    let mut queue: Vec<QueueTrack> = Vec::new();
    for aid in &album_ids {
        match runtime.core().get_album(aid.as_str()).await {
            Ok(album) => {
                let album_title = album.title.clone();
                let album_artist = album.artist.name.clone();
                let album_artwork = album.image.best().cloned().unwrap_or_default();
                let album_primary = Some(album.artist.id);
                let raw_tracks = album.tracks.as_ref().map(|c| c.items.as_slice()).unwrap_or_default();
                for track in raw_tracks {
                    if track_is_blacklisted_full(track, album_primary) {
                        continue;
                    }
                    queue.push(make_queue_track(
                        track,
                        &album.id,
                        &album_title,
                        &album_artist,
                        &album_artwork,
                        album.version.as_deref(),
                    ));
                }
            }
            Err(e) => {
                log::warn!("[qbz-slint] artist-play: get_album {aid} failed: {e}; skipping");
            }
        }
    }
    if queue.is_empty() {
        log::warn!(
            "[qbz-slint] artist-play: {artist_id} studio discography produced no playable tracks"
        );
        return None;
    }
    Some(queue)
}
