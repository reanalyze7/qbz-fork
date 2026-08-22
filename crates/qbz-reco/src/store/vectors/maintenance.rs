//! Freshness checks and cleanup of stored vectors.

use super::super::{current_timestamp, ArtistVectorStore};
use rusqlite::{params, OptionalExtension};

impl ArtistVectorStore {
    /// Check if a vector exists and is fresh (within TTL)
    pub fn has_fresh_vector(&self, mbid: &str, max_age_secs: i64) -> bool {
        let Some(artist_idx) = self.get_idx(mbid) else {
            return false;
        };

        let cutoff = current_timestamp() - max_age_secs;

        let result: Option<i64> = self
            .conn
            .query_row(
                "SELECT updated_at FROM vector_metadata WHERE artist_idx = ?1 AND updated_at > ?2",
                params![artist_idx, cutoff],
                |row| row.get(0),
            )
            .optional()
            .ok()
            .flatten();

        result.is_some()
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&mut self, max_age_secs: i64) -> Result<usize, String> {
        let cutoff = current_timestamp() - max_age_secs;

        let deleted = self
            .conn
            .execute(
                "DELETE FROM vector_entries WHERE updated_at < ?1",
                params![cutoff],
            )
            .map_err(|e| format!("Failed to delete expired entries: {}", e))?;

        // Also clean up metadata
        self.conn
            .execute(
                "DELETE FROM vector_metadata WHERE updated_at < ?1",
                params![cutoff],
            )
            .map_err(|e| format!("Failed to delete expired metadata: {}", e))?;

        Ok(deleted)
    }

    /// Clear all data from the store
    pub fn clear_all(&mut self) -> Result<usize, String> {
        let deleted = self
            .conn
            .execute("DELETE FROM vector_entries", [])
            .map_err(|e| format!("Failed to delete vector entries: {}", e))?;

        self.conn
            .execute("DELETE FROM vector_metadata", [])
            .map_err(|e| format!("Failed to delete metadata: {}", e))?;

        self.conn
            .execute("DELETE FROM artist_index", [])
            .map_err(|e| format!("Failed to delete artist index: {}", e))?;

        // Reset in-memory state
        self.artist_to_idx.clear();
        self.idx_to_artist.clear();
        self.next_idx = 0;

        log::info!("Artist vector store cleared: {} entries deleted", deleted);
        Ok(deleted)
    }
}
