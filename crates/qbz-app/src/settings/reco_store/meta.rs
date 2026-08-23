use rusqlite::params;
use std::sync::{Arc, Mutex};

use super::now_ts;
use super::schema::RecoStore;

impl RecoStore {
    /// Upsert an album-meta row (only needed so `get_top_genres` can resolve a
    /// genre name; mirrors the relevant columns of Tauri's `set_album_meta`).
    pub fn set_album_genre_name(&self, album_id: &str, genre_name: &str) -> Result<(), String> {
        self.conn
            .execute(
                r#"INSERT INTO reco_album_meta
                       (album_id, title, artist_name, genre_name, updated_at)
                   VALUES (?, '', '', ?, ?)
                   ON CONFLICT(album_id) DO UPDATE SET genre_name = excluded.genre_name"#,
                params![album_id, genre_name, now_ts()],
            )
            .map_err(|e| format!("Failed to upsert album genre meta: {}", e))?;
        Ok(())
    }

    /// Backfill `genre_id` onto every still-NULL event of an album once its
    /// genre is known (ported from Tauri `db.rs:321-331`). Plays log
    /// `genre_id = None`, so the frontend calls this when it resolves an
    /// album's genre — this is what makes `get_top_genres` non-empty.
    pub fn update_genre_for_album(&self, album_id: &str, genre_id: u64) -> Result<u64, String> {
        let affected = self
            .conn
            .execute(
                "UPDATE reco_events SET genre_id = ? WHERE album_id = ? AND genre_id IS NULL",
                params![genre_id, album_id],
            )
            .map_err(|e| format!("Failed to update genre for album: {}", e))?;
        Ok(affected as u64)
    }
}

pub type RecoStoreState = Arc<Mutex<Option<RecoStore>>>;

pub fn create_empty_reco_store_state() -> RecoStoreState {
    Arc::new(Mutex::new(None))
}
