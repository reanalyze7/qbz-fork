use std::sync::atomic::Ordering;

use rusqlite::params;

use super::{BlacklistService, BlacklistedAlbum};

impl BlacklistService {
    // ----- Album axis (String-keyed, shares the `enabled` flag) -----
    //
    // The shared enable/disable flag itself (`set_enabled`/`is_enabled`) lives
    // in `flags.rs` — this axis only ever READS `self.enabled`.

    /// Check if an album is blacklisted - O(1) operation.
    ///
    /// Returns false if the (shared) feature flag is disabled.
    #[inline]
    pub fn is_album_blacklisted(&self, album_id: &str) -> bool {
        if !self.enabled.load(Ordering::Relaxed) {
            return false;
        }
        self.blacklisted_album_ids
            .read()
            .map(|set| set.contains(album_id))
            .unwrap_or(false)
    }

    /// Add an album to the blacklist.
    pub fn add_album(
        &self,
        album_id: &str,
        album_title: &str,
        artist_name: &str,
        cover_url: &str,
        notes: Option<&str>,
    ) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO album_blacklist
                 (album_id, album_title, artist_name, cover_url, added_at, notes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![album_id, album_title, artist_name, cover_url, now, notes],
            )
            .map_err(|e| format!("Failed to add album to blacklist: {}", e))?;

        if let Ok(mut set) = self.blacklisted_album_ids.write() {
            set.insert(album_id.to_string());
        }

        log::info!(
            "[Blacklist] Added album: {} (id={})",
            album_title,
            album_id
        );
        Ok(())
    }

    /// Remove an album from the blacklist.
    pub fn remove_album(&self, album_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM album_blacklist WHERE album_id = ?1",
                params![album_id],
            )
            .map_err(|e| format!("Failed to remove album from blacklist: {}", e))?;

        if let Ok(mut set) = self.blacklisted_album_ids.write() {
            set.remove(album_id);
        }

        log::info!("[Blacklist] Removed album id={}", album_id);
        Ok(())
    }

    /// Get all blacklisted albums, ordered by title (case-insensitive).
    pub fn get_all_albums(&self) -> Result<Vec<BlacklistedAlbum>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT album_id, album_title, artist_name, cover_url, added_at, notes
                 FROM album_blacklist
                 ORDER BY album_title COLLATE NOCASE",
            )
            .map_err(|e| format!("Failed to prepare album query: {}", e))?;

        let albums = stmt
            .query_map([], |row| {
                Ok(BlacklistedAlbum {
                    album_id: row.get(0)?,
                    album_title: row.get(1)?,
                    artist_name: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    cover_url: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    added_at: row.get(4)?,
                    notes: row.get(5)?,
                })
            })
            .map_err(|e| format!("Failed to query album blacklist: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(albums)
    }
}
