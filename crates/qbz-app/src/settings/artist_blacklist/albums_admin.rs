use super::BlacklistService;

impl BlacklistService {
    /// Get count of blacklisted albums.
    ///
    /// Does not respect the enabled flag.
    pub fn album_count(&self) -> usize {
        self.blacklisted_album_ids
            .read()
            .map(|set| set.len())
            .unwrap_or(0)
    }

    /// Clear all blacklisted albums.
    ///
    /// Does not touch the settings row nor the artist table.
    pub fn clear_all_albums(&self) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM album_blacklist", [])
            .map_err(|e| format!("Failed to clear album blacklist: {}", e))?;

        if let Ok(mut set) = self.blacklisted_album_ids.write() {
            set.clear();
        }

        log::info!("[Blacklist] Cleared all album entries");
        Ok(())
    }
}
