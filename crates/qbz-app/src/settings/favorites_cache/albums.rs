use super::FavoritesCacheStore;
use rusqlite::params;

impl FavoritesCacheStore {
    // ============ Album favorites ============

    pub fn get_favorite_album_ids(&self) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT album_id FROM favorite_albums")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to query favorite albums: {}", e))?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
        }
        Ok(ids)
    }

    pub fn is_album_favorite(&self, album_id: &str) -> Result<bool, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM favorite_albums WHERE album_id = ?1")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let exists = stmt
            .exists(params![album_id])
            .map_err(|e| format!("Failed to check favorite: {}", e))?;

        Ok(exists)
    }

    pub fn add_favorite_album(&self, album_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO favorite_albums (album_id) VALUES (?1)",
                params![album_id],
            )
            .map_err(|e| format!("Failed to add favorite album: {}", e))?;
        Ok(())
    }

    pub fn remove_favorite_album(&self, album_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM favorite_albums WHERE album_id = ?1",
                params![album_id],
            )
            .map_err(|e| format!("Failed to remove favorite album: {}", e))?;
        Ok(())
    }

    pub fn sync_favorite_albums(&self, album_ids: &[String]) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM favorite_albums", [])
            .map_err(|e| format!("Failed to clear favorite albums: {}", e))?;

        for album_id in album_ids {
            self.conn
                .execute(
                    "INSERT INTO favorite_albums (album_id) VALUES (?1)",
                    params![album_id],
                )
                .map_err(|e| format!("Failed to insert favorite album: {}", e))?;
        }
        Ok(())
    }
}
