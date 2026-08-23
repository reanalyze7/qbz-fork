use super::FavoritesCacheStore;
use rusqlite::params;

impl FavoritesCacheStore {
    // ============ Artist favorites ============

    pub fn get_favorite_artist_ids(&self) -> Result<Vec<i64>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT artist_id FROM favorite_artists")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to query favorite artists: {}", e))?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
        }
        Ok(ids)
    }

    pub fn is_artist_favorite(&self, artist_id: i64) -> Result<bool, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM favorite_artists WHERE artist_id = ?1")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let exists = stmt
            .exists(params![artist_id])
            .map_err(|e| format!("Failed to check favorite: {}", e))?;

        Ok(exists)
    }

    pub fn add_favorite_artist(&self, artist_id: i64) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO favorite_artists (artist_id) VALUES (?1)",
                params![artist_id],
            )
            .map_err(|e| format!("Failed to add favorite artist: {}", e))?;
        Ok(())
    }

    pub fn remove_favorite_artist(&self, artist_id: i64) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM favorite_artists WHERE artist_id = ?1",
                params![artist_id],
            )
            .map_err(|e| format!("Failed to remove favorite artist: {}", e))?;
        Ok(())
    }

    pub fn sync_favorite_artists(&self, artist_ids: &[i64]) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM favorite_artists", [])
            .map_err(|e| format!("Failed to clear favorite artists: {}", e))?;

        for &artist_id in artist_ids {
            self.conn
                .execute(
                    "INSERT INTO favorite_artists (artist_id) VALUES (?1)",
                    params![artist_id],
                )
                .map_err(|e| format!("Failed to insert favorite artist: {}", e))?;
        }
        Ok(())
    }
}
