//! Pure cover/album-id lookups over a `qbz_models::Track`.

/// Best collage cover URL for a track's album (large → best variant).
pub(super) fn track_album_cover(track: &qbz_models::Track) -> Option<String> {
    track
        .album
        .as_ref()
        .and_then(|a| a.image.best().cloned())
        .filter(|s| !s.is_empty())
}

/// Album id of a track (for distinct-cover dedupe in the book collage).
pub(super) fn track_album_id(track: &qbz_models::Track) -> Option<String> {
    track
        .album
        .as_ref()
        .map(|a| a.id.clone())
        .filter(|s| !s.is_empty())
}
