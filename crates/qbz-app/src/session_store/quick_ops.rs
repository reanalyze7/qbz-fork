use rusqlite::params;
use std::time::{SystemTime, UNIX_EPOCH};

use super::schema::SessionStore;

impl SessionStore {
    pub fn save_position(&self, position_secs: u64) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn
            .execute(
                "UPDATE player_state SET current_position_secs = ?1, saved_at = ?2 WHERE id = 1",
                params![position_secs as i64, now],
            )
            .map_err(|e| format!("Failed to save position: {}", e))?;

        Ok(())
    }

    pub fn save_volume(&self, volume: f32) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE player_state SET volume = ?1 WHERE id = 1",
                params![volume as f64],
            )
            .map_err(|e| format!("Failed to save volume: {}", e))?;

        Ok(())
    }

    pub fn save_playback_mode(&self, shuffle: bool, repeat_mode: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE player_state SET shuffle_enabled = ?1, repeat_mode = ?2 WHERE id = 1",
                params![shuffle as i64, repeat_mode],
            )
            .map_err(|e| format!("Failed to save playback mode: {}", e))?;

        Ok(())
    }

    pub fn clear_session(&self) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM queue_tracks", [])
            .map_err(|e| format!("Failed to clear queue: {}", e))?;

        self.conn.execute(
            "UPDATE player_state SET current_index = NULL, current_position_secs = 0, was_playing = 0, last_view = 'home', view_context_id = NULL, view_context_type = NULL WHERE id = 1",
            [],
        ).map_err(|e| format!("Failed to reset player state: {}", e))?;

        Ok(())
    }
}
