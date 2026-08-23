//! Album <-> selected-artist credit matching.

use super::normalize::{normalize_artist, split_credit};

/// Does this album credit the (normalized) selected artist — as primary, in
/// `all_artists`, or as one part of a multi-artist credit? Mirrors Tauri's
/// `selectedArtistAlbums` predicate.
pub(crate) fn album_matches_artist(al: &qbz_library::LocalAlbum, nsel: &str) -> bool {
    if nsel == "various artists" {
        return normalize_artist(&al.artist) == "various artists";
    }
    if normalize_artist(&al.artist) == nsel {
        return true;
    }
    for part in al.all_artists.split(',') {
        if normalize_artist(part) == nsel {
            return true;
        }
    }
    for part in split_credit(&al.artist) {
        if normalize_artist(&part) == nsel {
            return true;
        }
    }
    false
}
