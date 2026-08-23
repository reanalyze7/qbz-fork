//! Mutations (Err with the Tauri "no active session" string when unbound).

use super::lifecycle::mutate;

/// Add an artist to the blacklist.
pub fn add(artist_id: u64, artist_name: &str, notes: Option<&str>) -> Result<(), String> {
    mutate(|s| s.add(artist_id, artist_name, notes))
}

/// Remove an artist from the blacklist.
pub fn remove(artist_id: u64) -> Result<(), String> {
    mutate(|s| s.remove(artist_id))
}

/// Toggle the global enable flag.
pub fn set_enabled(enabled: bool) -> Result<(), String> {
    mutate(|s| s.set_enabled(enabled))
}

/// Clear all blacklisted artists (leaves the enabled flag + albums untouched).
pub fn clear_all() -> Result<(), String> {
    mutate(|s| s.clear_all())
}

/// Add an album to the blacklist.
pub fn add_album(
    album_id: &str,
    album_title: &str,
    artist_name: &str,
    cover_url: &str,
    notes: Option<&str>,
) -> Result<(), String> {
    mutate(|s| s.add_album(album_id, album_title, artist_name, cover_url, notes))
}

/// Remove an album from the blacklist.
pub fn remove_album(album_id: &str) -> Result<(), String> {
    mutate(|s| s.remove_album(album_id))
}

/// Clear all blocked albums (leaves the enabled flag + artists untouched).
pub fn clear_all_albums() -> Result<(), String> {
    mutate(|s| s.clear_all_albums())
}
