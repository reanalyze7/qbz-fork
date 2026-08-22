use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Get custom track order for a playlist
    /// Returns Vec of (track_id, is_local, custom_position)
    pub fn get_playlist_custom_order(
        &self,
        qobuz_playlist_id: u64,
    ) -> Result<Vec<(i64, bool, i32)>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT track_id, is_local, custom_position
             FROM playlist_track_custom_order
             WHERE qobuz_playlist_id = ?1
             ORDER BY custom_position ASC",
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to prepare custom order query: {}", e))
            })?;

        let rows = stmt
            .query_map(params![qobuz_playlist_id as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i32>(1)? != 0,
                    row.get::<_, i32>(2)?,
                ))
            })
            .map_err(|e| LibraryError::Database(format!("Failed to query custom order: {}", e)))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| {
                LibraryError::Database(format!("Failed to read custom order row: {}", e))
            })?);
        }
        Ok(result)
    }

    /// Initialize custom order for a playlist from a list of track IDs
    /// This sets up the initial order based on the current track arrangement
    pub fn init_playlist_custom_order(
        &self,
        qobuz_playlist_id: u64,
        track_ids: &[(i64, bool)], // (track_id, is_local)
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

        for (position, (track_id, is_local)) in track_ids.iter().enumerate() {
            stmt.execute(params![
                qobuz_playlist_id as i64,
                *track_id,
                *is_local as i32,
                position as i32,
                now,
                now,
            ])
            .map_err(|e| LibraryError::Database(format!("Failed to insert custom order: {}", e)))?;
        }

        Ok(())
    }

    /// Check if a playlist has custom order defined
    pub fn has_playlist_custom_order(&self, qobuz_playlist_id: u64) -> Result<bool, LibraryError> {
        let count: i32 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM playlist_track_custom_order WHERE qobuz_playlist_id = ?1",
                params![qobuz_playlist_id as i64],
                |row| row.get(0),
            )
            .map_err(|e| LibraryError::Database(format!("Failed to check custom order: {}", e)))?;

        Ok(count > 0)
    }
}
