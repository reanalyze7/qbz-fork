use std::sync::atomic::Ordering;

use rusqlite::params;

use super::{BlacklistService, BlacklistedArtist};

impl BlacklistService {
    /// Check if an artist is blacklisted - O(1) operation.
    ///
    /// Returns false if the feature is disabled.
    #[inline]
    pub fn is_blacklisted(&self, artist_id: u64) -> bool {
        // Fast path: if feature is disabled, always return false.
        if !self.enabled.load(Ordering::Relaxed) {
            return false;
        }

        // O(1) HashSet lookup.
        self.blacklisted_ids
            .read()
            .map(|set| set.contains(&artist_id))
            .unwrap_or(false)
    }

    /// Add an artist to the blacklist.
    pub fn add(
        &self,
        artist_id: u64,
        artist_name: &str,
        notes: Option<&str>,
    ) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO artist_blacklist (artist_id, artist_name, added_at, notes)
                 VALUES (?1, ?2, ?3, ?4)",
                params![artist_id as i64, artist_name, now, notes],
            )
            .map_err(|e| format!("Failed to add artist to blacklist: {}", e))?;

        // Update in-memory set.
        if let Ok(mut set) = self.blacklisted_ids.write() {
            set.insert(artist_id);
        }

        log::info!(
            "[Blacklist] Added artist: {} (id={})",
            artist_name,
            artist_id
        );
        Ok(())
    }

    /// Remove an artist from the blacklist.
    pub fn remove(&self, artist_id: u64) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM artist_blacklist WHERE artist_id = ?1",
                params![artist_id as i64],
            )
            .map_err(|e| format!("Failed to remove artist from blacklist: {}", e))?;

        // Update in-memory set.
        if let Ok(mut set) = self.blacklisted_ids.write() {
            set.remove(&artist_id);
        }

        log::info!("[Blacklist] Removed artist id={}", artist_id);
        Ok(())
    }

    /// Get all blacklisted artists.
    pub fn get_all(&self) -> Result<Vec<BlacklistedArtist>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT artist_id, artist_name, added_at, notes
                 FROM artist_blacklist
                 ORDER BY artist_name COLLATE NOCASE",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let artists = stmt
            .query_map([], |row| {
                Ok(BlacklistedArtist {
                    artist_id: row.get::<_, i64>(0)? as u64,
                    artist_name: row.get(1)?,
                    added_at: row.get(2)?,
                    notes: row.get(3)?,
                })
            })
            .map_err(|e| format!("Failed to query blacklist: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(artists)
    }

    /// Get count of blacklisted artists.
    ///
    /// Does not respect the enabled flag.
    pub fn count(&self) -> usize {
        self.blacklisted_ids
            .read()
            .map(|set| set.len())
            .unwrap_or(0)
    }

    /// Clear all blacklisted artists.
    ///
    /// Does not touch the settings row.
    pub fn clear_all(&self) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM artist_blacklist", [])
            .map_err(|e| format!("Failed to clear blacklist: {}", e))?;

        if let Ok(mut set) = self.blacklisted_ids.write() {
            set.clear();
        }

        log::info!("[Blacklist] Cleared all entries");
        Ok(())
    }
}
