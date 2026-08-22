//! D-FEAT blacklist predicates: per-item true/false checks used by
//! search, discovery, and queue-build call sites.

use qbz_models::{Album, DiscoverAlbum, Track};

use super::super::{AlbumBlacklistFilter, BlacklistFilter};

/// D-FEAT: returns true if the album should be hidden by the blacklist.
///
/// Extends the historical Tauri rule (which blocked only the PRIMARY
/// `album.artist`) to also block when ANY contributor in `album.artists[]`
/// (featured artists included) is blacklisted. Centralizing this here keeps
/// every call site (search, discovery, queue-build) on ONE consistent rule.
///
/// Fail-open: an empty filter never blocks; an album with no matching id is
/// kept.
pub fn album_blacklisted(album: &Album, bl: &BlacklistFilter, album_bl: &AlbumBlacklistFilter) -> bool {
    if bl.is_empty() && album_bl.is_empty() {
        return false;
    }
    // Album axis (orthogonal): the album's OWN id being blocked hides it
    // regardless of artist.
    if album_bl.contains(&album.id) {
        return true;
    }
    if bl.contains(&album.artist.id) {
        return true;
    }
    album
        .artists
        .as_ref()
        .is_some_and(|v| v.iter().any(|a| bl.contains(&a.id)))
}

/// D-FEAT: returns true if the track should be hidden by the blacklist.
///
/// Blocks on the track's structured `performer` OR `composer` id. Extends the
/// historical Tauri rule (performer only) to also cover the composer.
///
/// Fail-open: an empty filter never blocks; a track with neither a performer
/// nor a composer id is kept (no id to match against).
///
/// D-FEAT limitation: the model exposes no structured per-track *featured
/// performer id* — only `performer`, `composer`, and a free-text `performers`
/// string. We deliberately do NOT name-match the free-text string; this rule
/// is strictly id-based.
pub fn track_blacklisted(track: &Track, bl: &BlacklistFilter, album_bl: &AlbumBlacklistFilter) -> bool {
    if bl.is_empty() && album_bl.is_empty() {
        return false;
    }
    // Album axis: a track of a blocked album is hidden too.
    if track
        .album
        .as_ref()
        .is_some_and(|a| album_bl.contains(&a.id))
    {
        return true;
    }
    track
        .performer
        .as_ref()
        .is_some_and(|a| bl.contains(&a.id))
        || track
            .composer
            .as_ref()
            .is_some_and(|a| bl.contains(&a.id))
}

/// D-FEAT: returns true if a discover-shaped album should be hidden.
///
/// Discover albums expose only a flat `artists[]` vec (no separate primary
/// `artist`), so any matching contributor id — primary or featured — blocks
/// the album. Fail-open: an empty filter never blocks.
pub fn discover_album_blacklisted(
    album: &DiscoverAlbum,
    bl: &BlacklistFilter,
    album_bl: &AlbumBlacklistFilter,
) -> bool {
    if bl.is_empty() && album_bl.is_empty() {
        return false;
    }
    if album_bl.contains(&album.id) {
        return true;
    }
    album.artists.iter().any(|a| bl.contains(&a.id))
}
