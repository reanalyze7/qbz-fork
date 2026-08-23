use crate::settings::search_cache::cache::page;
use qbz_models::{Album, Artist, Playlist, SearchAllResults, Track};
use std::path::PathBuf;

pub(super) fn unique_test_dir(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

// Album/Track do not derive Default; build them via serde from a minimal
// object (all their fields are Option or #[serde(default)]).
pub(super) fn album(id: u64) -> Album {
    serde_json::from_value(serde_json::json!({ "id": id.to_string() })).unwrap()
}

pub(super) fn track(id: u64) -> Track {
    serde_json::from_value(serde_json::json!({ "id": id })).unwrap()
}

pub(super) fn playlist(id: u64) -> Playlist {
    serde_json::from_value(serde_json::json!({ "id": id })).unwrap()
}

pub(super) fn artist(id: u64) -> Artist {
    Artist {
        id,
        ..Default::default()
    }
}

pub(super) fn results(
    albums: Vec<Album>,
    tracks: Vec<Track>,
    artists: Vec<Artist>,
    playlists: Vec<Playlist>,
) -> SearchAllResults {
    SearchAllResults {
        albums: page(albums),
        tracks: page(tracks),
        artists: page(artists),
        playlists: page(playlists),
        most_popular: None,
    }
}
