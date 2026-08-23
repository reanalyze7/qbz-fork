//! Artists-tab caches: the loaded album set (right-pane filter source) and
//! the pending-artist-to-select queue.

/// The loaded album set, cached so the right-pane filter (select) doesn't
/// re-hit the DB — mirrors Tauri filtering its in-memory `albums` array.
pub(crate) static ARTIST_ALBUMS: std::sync::Mutex<Vec<qbz_library::LocalAlbum>> =
    std::sync::Mutex::new(Vec::new());

/// An artist name to auto-select once the Artists tab finishes loading — set
/// when navigating to a local artist from outside the tab (LocalAlbum header
/// link, now-playing "Go to artist", a track's context menu). Consumed once.
static PENDING_ARTIST: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// Queue an artist to be selected as soon as the Artists tab is ready.
pub fn set_pending_artist(name: String) {
    *PENDING_ARTIST.lock().unwrap_or_else(|e| e.into_inner()) = Some(name);
}

pub(crate) fn take_pending_artist() -> Option<String> {
    PENDING_ARTIST.lock().unwrap_or_else(|e| e.into_inner()).take()
}

/// Album-identity mode changed: invalidate ONLY the Artists tab (its album
/// cache + right pane depend on the group key — a folder-mode compilation
/// cross-lists under every artist). The Albums tab reloads separately via
/// `reload_albums`; tracks/folders don't depend on album identity.
pub fn invalidate_artists(window: &crate::AppWindow) {
    use slint::{ComponentHandle, ModelRc, VecModel};
    let s = window.global::<crate::LocalLibraryState>();
    s.set_artists(ModelRc::new(VecModel::from(Vec::<crate::LocalArtistItem>::new())));
    s.set_artists_loading(false);
    if let Ok(mut cache) = ARTIST_ALBUMS.lock() {
        cache.clear();
    }
}
