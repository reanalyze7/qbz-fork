use super::FavoritesCacheStore;
use rusqlite::params;

impl FavoritesCacheStore {
    // ============ Label favorites ============

    pub fn get_favorite_label_ids(&self) -> Result<Vec<i64>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT label_id FROM favorite_labels")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to query favorite labels: {}", e))?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
        }
        Ok(ids)
    }

    pub fn is_label_favorite(&self, label_id: i64) -> Result<bool, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM favorite_labels WHERE label_id = ?1")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let exists = stmt
            .exists(params![label_id])
            .map_err(|e| format!("Failed to check favorite: {}", e))?;

        Ok(exists)
    }

    pub fn add_favorite_label(&self, label_id: i64) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO favorite_labels (label_id) VALUES (?1)",
                params![label_id],
            )
            .map_err(|e| format!("Failed to add favorite label: {}", e))?;
        Ok(())
    }

    pub fn remove_favorite_label(&self, label_id: i64) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM favorite_labels WHERE label_id = ?1",
                params![label_id],
            )
            .map_err(|e| format!("Failed to remove favorite label: {}", e))?;
        Ok(())
    }

    pub fn sync_favorite_labels(&self, label_ids: &[i64]) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM favorite_labels", [])
            .map_err(|e| format!("Failed to clear favorite labels: {}", e))?;

        for &label_id in label_ids {
            self.conn
                .execute(
                    "INSERT INTO favorite_labels (label_id) VALUES (?1)",
                    params![label_id],
                )
                .map_err(|e| format!("Failed to insert favorite label: {}", e))?;
        }
        Ok(())
    }
}
