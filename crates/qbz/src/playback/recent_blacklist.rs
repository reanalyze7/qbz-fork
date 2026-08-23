//! Queue-drop blacklist predicates shared by every queue-building path.

use qbz_models::{QueueTrack, Track};

/// THE single queue-drop predicate for an already-built `QueueTrack` (Task 7).
/// Returns `true` when the track must be removed from a play/shuffle/queue-next/
/// queue-later builder. Delegates to `artist_blacklist::is_track_blacklisted`,
/// the SAME underlying source-guard + per-id check the row greyout
/// (`stamp_row`) uses — so the queue can never diverge from the rendered list.
///
/// `QueueTrack` carries `source` + `artist_id` (performer) but NOT a composer
/// id, so this leg is performer-only. Builders that still hold the full
/// catalog `Track` (album / playlist / artist-top) ALSO filter at the `Track`
/// level via `track_is_blacklisted_full` below, which adds the composer leg
/// (D-FEAT). Local / no-id tracks => kept (fail-open).
pub(super) fn queue_track_blacklisted(track: &QueueTrack) -> bool {
    let source = track.source.as_deref().unwrap_or("qobuz");
    crate::artist_blacklist::is_track_blacklisted(
        source,
        track.artist_id,
        None,
        track.album_id.as_deref(),
    )
}

/// Drop blacklisted entries from a freshly-built `QueueTrack` queue. Keeps
/// local / no-id tracks (fail-open). The single filter every builder
/// applies before handing the queue to the core.
pub(super) fn filter_blacklisted_queue(queue: Vec<QueueTrack>) -> Vec<QueueTrack> {
    queue
        .into_iter()
        .filter(|t| !queue_track_blacklisted(t))
        .collect()
}

/// `Track`-level drop predicate (performer OR composer — full D-FEAT), for
/// builders that still hold the catalog `Track` before mapping to QueueTrack.
/// `album_primary` is the album's primary-artist id used as the row fallback
/// when the track carries no performer (album surfaces only — mirror the album
/// row stamp `track.artist_id ?? album.artist_id`). Always treated as Qobuz
/// (these builders only run on Qobuz catalog tracks; local play paths are
/// separate). Shares the underlying `is_blacklisted` check with the row stamp.
pub(super) fn track_is_blacklisted_full(track: &Track, album_primary: Option<u64>) -> bool {
    let performer = track.performer.as_ref().map(|p| p.id).or(album_primary);
    let composer = track.composer.as_ref().map(|c| c.id);
    crate::artist_blacklist::is_track_blacklisted(
        "qobuz",
        performer,
        composer,
        track.album.as_ref().map(|a| a.id.as_str()),
    )
}
