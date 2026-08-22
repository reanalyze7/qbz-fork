//! MBID <-> integer index lookups and creation.

use super::ArtistVectorStore;
use rusqlite::{params, OptionalExtension};

impl ArtistVectorStore {
    /// Get or create an index for an artist MBID
    pub fn get_or_create_idx(&mut self, mbid: &str, name: Option<&str>) -> Result<u32, String> {
        if let Some(&idx) = self.artist_to_idx.get(mbid) {
            return Ok(idx);
        }

        let idx = self.next_idx;
        self.next_idx += 1;

        self.conn
            .execute(
                "INSERT INTO artist_index (idx, mbid, name) VALUES (?1, ?2, ?3)",
                params![idx, mbid, name],
            )
            .map_err(|e| format!("Failed to insert artist index: {}", e))?;

        self.artist_to_idx.insert(mbid.to_string(), idx);

        // Extend idx_to_artist
        while self.idx_to_artist.len() <= idx as usize {
            self.idx_to_artist.push(String::new());
        }
        self.idx_to_artist[idx as usize] = mbid.to_string();

        Ok(idx)
    }

    /// Get index for an artist MBID (returns None if not found)
    pub fn get_idx(&self, mbid: &str) -> Option<u32> {
        self.artist_to_idx.get(mbid).copied()
    }

    /// Get MBID for an index
    pub fn get_mbid(&self, idx: u32) -> Option<&str> {
        self.idx_to_artist.get(idx as usize).map(|s| s.as_str())
    }

    /// Get artist name from index
    pub fn get_artist_name(&self, mbid: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT name FROM artist_index WHERE mbid = ?1",
                params![mbid],
                |row| row.get(0),
            )
            .optional()
            .ok()
            .flatten()
    }
}
