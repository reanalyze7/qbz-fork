use crate::LocalTrack;

/// Returns `Some(v)` iff every non-blank track shares one
/// `album_artist ?? artist`, else `None`. Empty / all-blank => `None`.
/// Port of the Tauri `library_compute_track_artist_match`.
pub fn compute_track_artist_match(tracks: &[LocalTrack]) -> Option<String> {
    let mut artists: std::collections::HashSet<String> = std::collections::HashSet::new();
    for track in tracks {
        let value = track
            .album_artist
            .as_deref()
            .unwrap_or(track.artist.as_str())
            .trim();
        if value.is_empty() {
            continue;
        }
        artists.insert(value.to_string());
        if artists.len() > 1 {
            return None;
        }
    }
    artists.into_iter().next()
}
