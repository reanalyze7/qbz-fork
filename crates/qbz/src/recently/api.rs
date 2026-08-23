//! Public read/write API — called by Discover Home and the playback session.

use super::model::{RecentAlbum, RecentTrack};
use super::store_io::{read_store, write_store, MAX_RECENT, MAX_RECENT_ALBUMS};

/// Load the recently-played tracks, newest first. Returns an empty list
/// when the store does not exist yet or cannot be read.
pub fn load() -> Vec<RecentTrack> {
    read_store().tracks
}

/// Recently-played albums, newest first, from the dedicated album history
/// (legacy stores derive it from the track window once — see `store_io::read_store`).
pub fn load_albums() -> Vec<RecentAlbum> {
    read_store().albums
}

/// Remove every recently-played entry whose `album_id` is in `album_ids`.
/// Used when a Local Library folder is deleted so its albums/tracks no longer
/// linger in Recently Played. Prunes BOTH the track and the album histories.
/// Returns how many track entries were removed.
pub fn prune_albums(album_ids: &[String]) -> usize {
    if album_ids.is_empty() {
        return 0;
    }
    let mut store = read_store();
    let tracks_before = store.tracks.len();
    let albums_before = store.albums.len();
    store.tracks.retain(|t| !album_ids.iter().any(|k| k == &t.album_id));
    store.albums.retain(|a| !album_ids.iter().any(|k| k == &a.id));
    let removed = tracks_before - store.tracks.len();
    if removed > 0 || albums_before != store.albums.len() {
        write_store(&store);
    }
    removed
}

/// Record a played track at the front of the track history (dedup by track
/// id, capped at `MAX_RECENT`) and its album at the front of the album
/// history (dedup by album id, capped at `MAX_RECENT_ALBUMS`). Called by
/// the playback session when a track starts.
#[allow(dead_code)] // wired by the playback session
pub fn record(track: RecentTrack) {
    let mut store = read_store();
    if !track.album_id.is_empty() {
        store.albums.retain(|a| a.id != track.album_id);
        store.albums.insert(
            0,
            RecentAlbum {
                id: track.album_id.clone(),
                title: track.album_title.clone(),
                artist: track.album_artist.clone(),
                artwork_url: track.album_artwork_url.clone(),
                quality_tier: track.quality_tier.clone(),
                quality_label: track.quality_label.clone(),
                genre: track.genre.clone(),
                release_date: track.release_date.clone(),
                source: track.source.clone(),
            },
        );
        store.albums.truncate(MAX_RECENT_ALBUMS);
    }
    store.tracks.retain(|t| t.id != track.id);
    store.tracks.insert(0, track);
    store.tracks.truncate(MAX_RECENT);
    write_store(&store);
}
