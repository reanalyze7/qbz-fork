use rusqlite::params;

use super::now_ts;
use super::schema::RecoStore;
use super::types::{RecoEventInput, RecoEventType, RecoItemType};

/// A decoded event row (mirrors `RecoEventRecord`).
#[derive(Debug, Clone)]
pub(super) struct RecoEventRecord {
    pub(super) event_type: String,
    pub(super) item_type: String,
    pub(super) track_id: Option<u64>,
    pub(super) album_id: Option<String>,
    pub(super) artist_id: Option<u64>,
    #[allow(dead_code)]
    pub(super) genre_id: Option<u64>,
    pub(super) created_at: i64,
}

impl RecoStore {
    // ---- Event logging ----

    /// Generic insert (mirrors `RecoStoreDb::insert_event`).
    pub fn insert_event(&self, event: &RecoEventInput) -> Result<(), String> {
        self.conn
            .execute(
                r#"
                INSERT INTO reco_events (
                    event_type, item_type, track_id, album_id,
                    artist_id, playlist_id, genre_id, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    event.event_type.as_str(),
                    event.item_type.as_str(),
                    event.track_id,
                    event.album_id.as_deref(),
                    event.artist_id,
                    event.playlist_id,
                    event.genre_id,
                    now_ts(),
                ],
            )
            .map_err(|e| format!("Failed to insert reco event: {}", e))?;
        Ok(())
    }

    /// Log a track play (event_type=play, item_type=track). Captures
    /// track_id + artist_id + genre_id + occurred_at (now).
    pub fn log_play_event(
        &self,
        track_id: u64,
        album_id: Option<String>,
        artist_id: Option<u64>,
        genre_id: Option<u64>,
    ) -> Result<(), String> {
        self.insert_event(&RecoEventInput {
            event_type: RecoEventType::Play,
            item_type: RecoItemType::Track,
            track_id: Some(track_id),
            album_id,
            artist_id,
            playlist_id: None,
            genre_id,
        })
    }

    /// Log a track favorite (event_type=favorite, item_type=track).
    pub fn log_favorite_event(
        &self,
        track_id: u64,
        album_id: Option<String>,
        artist_id: Option<u64>,
        genre_id: Option<u64>,
    ) -> Result<(), String> {
        self.insert_event(&RecoEventInput {
            event_type: RecoEventType::Favorite,
            item_type: RecoItemType::Track,
            track_id: Some(track_id),
            album_id,
            artist_id,
            playlist_id: None,
            genre_id,
        })
    }

    pub(super) fn get_events_since(
        &self,
        since_ts: i64,
        limit: u32,
    ) -> Result<Vec<RecoEventRecord>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT event_type, item_type, track_id, album_id, artist_id, genre_id, created_at
                FROM reco_events
                WHERE created_at >= ?
                ORDER BY created_at DESC
                LIMIT ?
                "#,
            )
            .map_err(|e| format!("Failed to prepare reco events query: {}", e))?;
        let rows = stmt
            .query_map(params![since_ts, limit], |row| {
                Ok(RecoEventRecord {
                    event_type: row.get(0)?,
                    item_type: row.get(1)?,
                    track_id: row.get(2)?,
                    album_id: row.get(3)?,
                    artist_id: row.get(4)?,
                    genre_id: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to query reco events: {}", e))?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|e| format!("Failed to read reco event row: {}", e))?);
        }
        Ok(events)
    }
}
