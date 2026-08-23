//! Pure Qobuz share-URL builders — no I/O.

/// Canonical Qobuz track URL — the `open.qobuz.com` share form (#514).
pub fn qobuz_track_url(track_id: &str) -> String {
    format!("https://open.qobuz.com/track/{track_id}")
}

/// Qobuz web-player playlist URL (matches Tauri's share-playlist link).
pub fn qobuz_playlist_url(playlist_id: &str) -> String {
    format!("https://play.qobuz.com/playlist/{playlist_id}")
}

/// Qobuz album URL — the `open.qobuz.com` form (#514; Tauri's
/// `shareAlbumQobuzLink` used `https://play.qobuz.com/album/{id}`). Also
/// the source URL fed to Song.link for the album-level "Album.link".
pub fn qobuz_album_url(album_id: &str) -> String {
    format!("https://open.qobuz.com/album/{album_id}")
}

/// Qobuz web-player artist URL (header Share action).
pub fn qobuz_artist_url(artist_id: &str) -> String {
    format!("https://play.qobuz.com/artist/{artist_id}")
}

/// Qobuz web-player label URL (label-page header Share action). There is no
/// Song.link/Album.link equivalent for labels — Qobuz-link only.
pub fn qobuz_label_url(label_id: &str) -> String {
    format!("https://play.qobuz.com/label/{label_id}")
}
