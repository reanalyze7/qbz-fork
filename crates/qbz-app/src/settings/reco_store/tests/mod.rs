mod basic;
mod genres_and_seeds;
mod train_and_agg;

use rusqlite::params;

use crate::settings::reco_store::RecoStore;

pub(super) fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

/// Insert an event at an explicit timestamp (test-only; the public log
/// helpers always stamp `now`).
#[allow(clippy::too_many_arguments)]
pub(super) fn insert_at(
    store: &RecoStore,
    event_type: &str,
    item_type: &str,
    track_id: Option<u64>,
    album_id: Option<&str>,
    artist_id: Option<u64>,
    genre_id: Option<u64>,
    created_at: i64,
) {
    store
        .conn
        .execute(
            r#"INSERT INTO reco_events
               (event_type, item_type, track_id, album_id, artist_id, playlist_id, genre_id, created_at)
               VALUES (?, ?, ?, ?, ?, NULL, ?, ?)"#,
            params![event_type, item_type, track_id, album_id, artist_id, genre_id, created_at],
        )
        .expect("insert event at ts");
}
