use rusqlite::params;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::session_store::model::PersistedSessionSnapshot;
use crate::session_store::schema::SessionStore;

impl SessionStore {
    pub fn save_session(&self, session: &PersistedSessionSnapshot) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn
            .execute("BEGIN TRANSACTION", [])
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        if let Err(e) = self.conn.execute("DELETE FROM queue_tracks", []) {
            let _ = self.conn.execute("ROLLBACK", []);
            return Err(format!("Failed to clear queue: {}", e));
        }

        for (pos, track) in session.playback.queue_tracks.iter().enumerate() {
            if let Err(e) = self.conn.execute(
                "INSERT INTO queue_tracks (position, track_id, title, artist, album, duration_secs, artwork_url, hires, bit_depth, sample_rate, is_local, album_id, artist_id, source, streamable, parental_warning, source_item_id_hint)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    pos as i64,
                    track.id as i64,
                    track.title,
                    track.artist,
                    track.album,
                    track.duration_secs as i64,
                    track.artwork_url,
                    track.hires as i64,
                    track.bit_depth.map(|v| v as i64),
                    track.sample_rate,
                    track.is_local as i64,
                    track.album_id,
                    track.artist_id.map(|v| v as i64),
                    track.source,
                    track.streamable as i64,
                    track.parental_warning as i64,
                    track.source_item_id_hint,
                ],
            ) {
                let _ = self.conn.execute("ROLLBACK", []);
                return Err(format!("Failed to insert queue track: {}", e));
            }
        }

        if let Err(e) = self.conn.execute(
            "UPDATE player_state SET
                current_index = ?1,
                current_position_secs = ?2,
                volume = ?3,
                shuffle_enabled = ?4,
                repeat_mode = ?5,
                was_playing = ?6,
                saved_at = ?7,
                last_view = ?8,
                view_context_id = ?9,
                view_context_type = ?10
             WHERE id = 1",
            params![
                session.playback.current_index.map(|i| i as i64),
                session.playback.current_position_secs as i64,
                session.playback.volume as f64,
                session.playback.shuffle_enabled as i64,
                session.playback.repeat_mode,
                session.playback.was_playing as i64,
                now,
                session.shell_view.last_view,
                session.shell_view.view_context_id,
                session.shell_view.view_context_type,
            ],
        ) {
            let _ = self.conn.execute("ROLLBACK", []);
            return Err(format!("Failed to update player state: {}", e));
        }

        self.conn
            .execute("COMMIT", [])
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(())
    }
}
