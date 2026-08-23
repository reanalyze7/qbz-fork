use super::FavoritesCacheStore;
use rusqlite::params;

impl FavoritesCacheStore {
    // ============ Track favorites ============

    pub fn get_favorite_track_ids(&self) -> Result<Vec<i64>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT track_id FROM favorite_tracks")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to query favorite tracks: {}", e))?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
        }
        Ok(ids)
    }

    pub fn is_track_favorite(&self, track_id: i64) -> Result<bool, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM favorite_tracks WHERE track_id = ?1")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let exists = stmt
            .exists(params![track_id])
            .map_err(|e| format!("Failed to check favorite: {}", e))?;

        Ok(exists)
    }

    pub fn add_favorite_track(&self, track_id: i64) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO favorite_tracks (track_id) VALUES (?1)",
                params![track_id],
            )
            .map_err(|e| format!("Failed to add favorite track: {}", e))?;
        Ok(())
    }

    pub fn remove_favorite_track(&self, track_id: i64) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM favorite_tracks WHERE track_id = ?1",
                params![track_id],
            )
            .map_err(|e| format!("Failed to remove favorite track: {}", e))?;
        Ok(())
    }

    pub fn sync_favorite_tracks(&self, track_ids: &[i64]) -> Result<(), String> {
        // Clear existing and insert new
        self.conn
            .execute("DELETE FROM favorite_tracks", [])
            .map_err(|e| format!("Failed to clear favorite tracks: {}", e))?;

        for &track_id in track_ids {
            self.conn
                .execute(
                    "INSERT INTO favorite_tracks (track_id) VALUES (?1)",
                    params![track_id],
                )
                .map_err(|e| format!("Failed to insert favorite track: {}", e))?;
        }
        Ok(())
    }
}
