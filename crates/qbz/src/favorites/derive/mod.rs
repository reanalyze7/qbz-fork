//! Per-tab derive: search/sort/group the loaded favorites into the
//! rendered `*-visible` (and `*-grouped`) models.

mod albums;
mod artists;
mod labels;
mod playlists;
mod tracks;

pub use albums::derive_albums;
pub use artists::derive_artists;
pub use labels::derive_labels;
pub use playlists::derive_playlists;
pub use tracks::derive_tracks;

/// First-letter bucket key for alpha grouping (# for non-alphabetic).
pub(crate) fn album_alpha_key(title: &str) -> String {
    match title.trim().chars().next() {
        Some(c) if c.is_ascii_alphabetic() => c.to_ascii_uppercase().to_string(),
        Some(c) if c.is_alphabetic() => c.to_uppercase().to_string(),
        _ => "#".to_string(),
    }
}

/// True if `genre` matches any selected genre name (favorites filter).
pub(crate) fn album_genre_matches(genre: &str, names: &[String]) -> bool {
    if names.is_empty() {
        return true;
    }
    let g = genre.to_lowercase();
    names.iter().any(|n| g.contains(&n.to_lowercase()))
}

/// Same, looking the track's genre up in the id->genre map.
pub(crate) fn track_genre_matches(id: &str, names: &[String]) -> bool {
    if names.is_empty() {
        return true;
    }
    crate::favorites::FAV_TRACK_GENRE.with(|m| {
        m.borrow()
            .get(id)
            .map(|g| {
                let gl = g.to_lowercase();
                names.iter().any(|n| gl.contains(&n.to_lowercase()))
            })
            .unwrap_or(false)
    })
}
