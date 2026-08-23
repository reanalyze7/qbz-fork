use slint::{ComponentHandle, Model};

use crate::{AppWindow, FavoritesState};

/// A random album id from the currently-visible set (Shuffle / random).
pub fn random_visible_album(window: &AppWindow) -> Option<String> {
    let model = window.global::<FavoritesState>().get_albums_visible();
    let n = model.row_count();
    if n == 0 {
        return None;
    }
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    let idx = (seed % n as u64) as usize;
    model.row_data(idx).map(|a| a.id.to_string())
}

/// A random artist id from the currently-visible favorites set. Tauri's
/// Artists header Shuffle opens a random ARTIST (not a random album).
pub fn random_visible_artist(window: &AppWindow) -> Option<String> {
    let model = window.global::<FavoritesState>().get_artists_visible();
    let n = model.row_count();
    if n == 0 {
        return None;
    }
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    let idx = (seed % n as u64) as usize;
    model.row_data(idx).map(|a| a.id.to_string())
}

/// A random playlist id from the currently-visible Playlists set (for the
/// Playlists "random" button — play a random playlist).
pub fn random_visible_playlist(window: &AppWindow) -> Option<String> {
    let model = window.global::<FavoritesState>().get_playlists_visible();
    let n = model.row_count();
    if n == 0 {
        return None;
    }
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    let idx = (seed % n as u64) as usize;
    model.row_data(idx).map(|p| p.id.to_string())
}

/// A random label (id, name) from the currently-visible Labels set (for the
/// Labels "random" button — open a random label's landing).
pub fn random_visible_label(window: &AppWindow) -> Option<(String, String)> {
    let model = window.global::<FavoritesState>().get_labels_visible();
    let n = model.row_count();
    if n == 0 {
        return None;
    }
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    let idx = (seed % n as u64) as usize;
    model
        .row_data(idx)
        .map(|l| (l.id.to_string(), l.name.to_string()))
}
