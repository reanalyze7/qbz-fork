//! Storing and retrieving sparse vectors.

use super::super::{current_timestamp, ArtistVectorStore};
use crate::sparse_vector::SparseVector;
use rusqlite::params;

impl ArtistVectorStore {
    /// Store a vector for an artist (delete-then-insert per `(artist, source)`).
    pub fn set_vector(
        &mut self,
        mbid: &str,
        vector: &SparseVector,
        source: &str,
    ) -> Result<(), String> {
        let artist_idx = self.get_or_create_idx(mbid, None)?;
        let now = current_timestamp();

        // Delete existing entries for this artist+source
        self.conn
            .execute(
                "DELETE FROM vector_entries WHERE artist_idx = ?1 AND source = ?2",
                params![artist_idx, source],
            )
            .map_err(|e| format!("Failed to delete old entries: {}", e))?;

        // Insert new entries
        let mut stmt = self
            .conn
            .prepare(
                "INSERT INTO vector_entries (artist_idx, target_idx, weight, source, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .map_err(|e| format!("Failed to prepare insert: {}", e))?;

        for (target_idx, weight) in vector.iter() {
            stmt.execute(params![artist_idx, target_idx, weight, source, now])
                .map_err(|e| format!("Failed to insert entry: {}", e))?;
        }

        // Update metadata
        self.conn
            .execute(
                "INSERT OR REPLACE INTO vector_metadata (artist_idx, updated_at, nnz)
                 VALUES (?1, ?2, ?3)",
                params![artist_idx, now, vector.nnz()],
            )
            .map_err(|e| format!("Failed to update metadata: {}", e))?;

        Ok(())
    }

    /// Get the combined vector for an artist (all sources merged)
    pub fn get_vector(&self, mbid: &str) -> Option<SparseVector> {
        let artist_idx = self.get_idx(mbid)?;

        let mut stmt = self
            .conn
            .prepare("SELECT target_idx, SUM(weight) FROM vector_entries WHERE artist_idx = ?1 GROUP BY target_idx")
            .ok()?;

        let rows = stmt
            .query_map(params![artist_idx], |row| {
                Ok((row.get::<_, u32>(0)?, row.get::<_, f32>(1)?))
            })
            .ok()?;

        let mut indices = Vec::new();
        let mut values = Vec::new();

        for row in rows.flatten() {
            indices.push(row.0);
            values.push(row.1);
        }

        if indices.is_empty() {
            return None;
        }

        // Sort by index
        let mut pairs: Vec<_> = indices.into_iter().zip(values).collect();
        pairs.sort_by_key(|(idx, _)| *idx);

        let (indices, values): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();

        Some(SparseVector::from_parts(indices, values))
    }
}
