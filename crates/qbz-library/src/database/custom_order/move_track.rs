use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Move a single track to a new position (reorders other tracks accordingly)
    pub fn move_playlist_track(
        &self,
        qobuz_playlist_id: u64,
        track_id: i64,
        is_local: bool,
        new_position: i32,
    ) -> Result<(), LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Get current position of the track
        let current_position: Option<i32> = self
            .conn
            .query_row(
                "SELECT custom_position FROM playlist_track_custom_order
             WHERE qobuz_playlist_id = ?1 AND track_id = ?2 AND is_local = ?3",
                params![qobuz_playlist_id as i64, track_id, is_local as i32],
                |row| row.get(0),
            )
            .ok();

        let current_position = match current_position {
            Some(pos) => pos,
            None => {
                // Track not in custom order yet, just insert it
                self.conn.execute(
                    "INSERT INTO playlist_track_custom_order
                     (qobuz_playlist_id, track_id, is_local, custom_position, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![qobuz_playlist_id as i64, track_id, is_local as i32, new_position, now, now],
                ).map_err(|e| LibraryError::Database(format!("Failed to insert track position: {}", e)))?;
                return Ok(());
            }
        };

        if current_position == new_position {
            return Ok(());
        }

        // Shift other tracks to make room
        if new_position < current_position {
            // Moving up: shift tracks between new_position and current_position down
            self.conn
                .execute(
                    "UPDATE playlist_track_custom_order
                 SET custom_position = custom_position + 1, updated_at = ?4
                 WHERE qobuz_playlist_id = ?1
                   AND custom_position >= ?2
                   AND custom_position < ?3",
                    params![
                        qobuz_playlist_id as i64,
                        new_position,
                        current_position,
                        now
                    ],
                )
                .map_err(|e| LibraryError::Database(format!("Failed to shift tracks: {}", e)))?;
        } else {
            // Moving down: shift tracks between current_position and new_position up
            self.conn
                .execute(
                    "UPDATE playlist_track_custom_order
                 SET custom_position = custom_position - 1, updated_at = ?4
                 WHERE qobuz_playlist_id = ?1
                   AND custom_position > ?2
                   AND custom_position <= ?3",
                    params![
                        qobuz_playlist_id as i64,
                        current_position,
                        new_position,
                        now
                    ],
                )
                .map_err(|e| LibraryError::Database(format!("Failed to shift tracks: {}", e)))?;
        }

        // Update the track's position
        self.conn
            .execute(
                "UPDATE playlist_track_custom_order
             SET custom_position = ?3, updated_at = ?5
             WHERE qobuz_playlist_id = ?1 AND track_id = ?2 AND is_local = ?4",
                params![
                    qobuz_playlist_id as i64,
                    track_id,
                    new_position,
                    is_local as i32,
                    now
                ],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to update track position: {}", e))
            })?;

        Ok(())
    }
}
