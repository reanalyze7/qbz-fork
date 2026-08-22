use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Set entire custom order for a playlist (batch update)
    pub fn set_playlist_custom_order(
        &self,
        qobuz_playlist_id: u64,
        orders: &[(i64, bool, i32)], // (track_id, is_local, position)
    ) -> Result<(), LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Clear existing custom order
        self.conn
            .execute(
                "DELETE FROM playlist_track_custom_order WHERE qobuz_playlist_id = ?1",
                params![qobuz_playlist_id as i64],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to clear existing custom order: {}", e))
            })?;

        // Insert new order
        let mut stmt = self
            .conn
            .prepare(
                "INSERT INTO playlist_track_custom_order
             (qobuz_playlist_id, track_id, is_local, custom_position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to prepare custom order insert: {}", e))
            })?;

        for (track_id, is_local, position) in orders {
            stmt.execute(params![
                qobuz_playlist_id as i64,
                *track_id,
                *is_local as i32,
                *position,
                now,
                now,
            ])
            .map_err(|e| LibraryError::Database(format!("Failed to insert custom order: {}", e)))?;
        }

        Ok(())
    }

    /// Clear custom order for a playlist
    pub fn clear_playlist_custom_order(&self, qobuz_playlist_id: u64) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "DELETE FROM playlist_track_custom_order WHERE qobuz_playlist_id = ?1",
                params![qobuz_playlist_id as i64],
            )
            .map_err(|e| LibraryError::Database(format!("Failed to clear custom order: {}", e)))?;

        Ok(())
    }
}
