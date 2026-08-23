//! File I/O + legacy-migration logic for the recently-played store.

use std::path::PathBuf;

use super::model::{RecentAlbum, RecentStore, RecentTrack};

/// How many recent tracks to keep.
pub(super) const MAX_RECENT: usize = 24;

/// How many recent albums to keep — independent of the track cap, so a
/// string of long albums cannot shrink the distinct-album history (#567).
pub(super) const MAX_RECENT_ALBUMS: usize = 24;

pub(super) fn store_path() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join("qbz").join("recently_played.json"))
}

/// Derive an album list from a track window by first-occurrence de-dup —
/// the pre-#567 behaviour, kept ONLY for the one-time legacy migration in
/// [`read_store`]. A track with no album id is skipped.
fn derive_albums(tracks: &[RecentTrack]) -> Vec<RecentAlbum> {
    let mut albums: Vec<RecentAlbum> = Vec::new();
    for track in tracks {
        if track.album_id.is_empty() || albums.iter().any(|a| a.id == track.album_id) {
            continue;
        }
        albums.push(RecentAlbum {
            id: track.album_id.clone(),
            title: track.album_title.clone(),
            artist: track.album_artist.clone(),
            artwork_url: track.album_artwork_url.clone(),
            quality_tier: track.quality_tier.clone(),
            quality_label: track.quality_label.clone(),
            genre: track.genre.clone(),
            release_date: track.release_date.clone(),
            source: track.source.clone(),
        });
    }
    albums
}

/// Read the whole store. Missing / unreadable file -> empty. A legacy bare
/// track array (pre-#567) migrates additively: the album list is derived from
/// the track window exactly as the old reader did; the next write persists
/// the new object shape.
pub(super) fn read_store() -> RecentStore {
    let Some(path) = store_path() else {
        return RecentStore::default();
    };
    let Ok(bytes) = std::fs::read(&path) else {
        return RecentStore::default();
    };
    if let Ok(store) = serde_json::from_slice::<RecentStore>(&bytes) {
        return store;
    }
    let tracks: Vec<RecentTrack> = serde_json::from_slice(&bytes).unwrap_or_default();
    let albums = derive_albums(&tracks);
    RecentStore { tracks, albums }
}

/// Persist the whole store (pretty JSON, best-effort with logged warnings).
pub(super) fn write_store(store: &RecentStore) {
    let Some(path) = store_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::warn!("[qbz-slint] recently-played store dir failed: {e}");
            return;
        }
    }
    match serde_json::to_vec_pretty(store) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                log::warn!("[qbz-slint] recently-played write failed: {e}");
            }
        }
        Err(e) => log::warn!("[qbz-slint] recently-played serialize failed: {e}"),
    }
}
