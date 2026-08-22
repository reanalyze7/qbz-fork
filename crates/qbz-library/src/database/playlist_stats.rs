//! Playlist play-count tracking, used to sort/surface frequently played
//! playlists.

use rusqlite::{params, OptionalExtension};

use crate::database::PlaylistStats;
use crate::LibraryError;

use super::LibraryDatabase;

impl LibraryDatabase {
    // === Playlist Stats ===

    /// Get playlist stats
    pub fn get_playlist_stats(
        &self,
        qobuz_playlist_id: u64,
    ) -> Result<Option<PlaylistStats>, LibraryError> {
        let result = self
            .conn
            .query_row(
                "SELECT qobuz_playlist_id, play_count, last_played_at, created_at, updated_at
             FROM playlist_stats WHERE qobuz_playlist_id = ?1",
                params![qobuz_playlist_id as i64],
                |row| {
                    Ok(PlaylistStats {
                        qobuz_playlist_id: row.get::<_, i64>(0)? as u64,
                        play_count: row.get::<_, i32>(1)? as u32,
                        last_played_at: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|e| LibraryError::Database(format!("Failed to get playlist stats: {}", e)))?;

        Ok(result)
    }

    /// Increment play count and update last_played_at for a playlist
    pub fn increment_playlist_play_count(
        &self,
        qobuz_playlist_id: u64,
    ) -> Result<PlaylistStats, LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Try to update existing, if none exists, insert new
        let existing = self.get_playlist_stats(qobuz_playlist_id)?;

        if let Some(mut stats) = existing {
            stats.play_count += 1;
            stats.last_played_at = Some(now);
            stats.updated_at = now;

            self.conn.execute(
                "UPDATE playlist_stats SET play_count = ?1, last_played_at = ?2, updated_at = ?3
                 WHERE qobuz_playlist_id = ?4",
                params![stats.play_count as i32, now, now, qobuz_playlist_id as i64],
            ).map_err(|e| LibraryError::Database(format!("Failed to increment play count: {}", e)))?;

            Ok(stats)
        } else {
            let stats = PlaylistStats {
                qobuz_playlist_id,
                play_count: 1,
                last_played_at: Some(now),
                created_at: now,
                updated_at: now,
            };

            self.conn.execute(
                "INSERT INTO playlist_stats (qobuz_playlist_id, play_count, last_played_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![qobuz_playlist_id as i64, 1, now, now, now],
            ).map_err(|e| LibraryError::Database(format!("Failed to create playlist stats: {}", e)))?;

            Ok(stats)
        }
    }

    /// Get all playlist stats (for sorting by play count)
    pub fn get_all_playlist_stats(&self) -> Result<Vec<PlaylistStats>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT qobuz_playlist_id, play_count, last_played_at, created_at, updated_at
             FROM playlist_stats ORDER BY play_count DESC",
            )
            .map_err(|e| LibraryError::Database(format!("Failed to prepare statement: {}", e)))?;

        let stats = stmt
            .query_map([], |row| {
                Ok(PlaylistStats {
                    qobuz_playlist_id: row.get::<_, i64>(0)? as u64,
                    play_count: row.get::<_, i32>(1)? as u32,
                    last_played_at: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .map_err(|e| {
                LibraryError::Database(format!("Failed to query playlist stats: {}", e))
            })?;

        stats
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| LibraryError::Database(format!("Failed to collect playlist stats: {}", e)))
    }
}
