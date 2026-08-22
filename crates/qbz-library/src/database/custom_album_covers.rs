//! Custom album cover overrides: lets a user pin a specific image path to
//! an album, taking precedence over whatever artwork the scanner/Qobuz
//! metadata would otherwise show.

use rusqlite::{params, OptionalExtension};

use crate::LibraryError;

use super::LibraryDatabase;

impl LibraryDatabase {
    /// Set a custom album cover
    pub fn set_custom_album_cover(
        &self,
        album_id: &str,
        custom_image_path: &str,
    ) -> Result<(), LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO custom_album_covers (album_id, custom_image_path, created_at)
                 VALUES (?1, ?2, ?3)",
                params![album_id, custom_image_path, now],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to set custom album cover: {}", e))
            })?;
        Ok(())
    }

    /// Get custom album cover path for a single album
    pub fn get_custom_album_cover(&self, album_id: &str) -> Result<Option<String>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT custom_image_path FROM custom_album_covers WHERE album_id = ?1")
            .map_err(|e| LibraryError::Database(format!("Failed to prepare query: {}", e)))?;

        let result = stmt
            .query_row(params![album_id], |row| row.get::<_, String>(0))
            .optional()
            .map_err(|e| {
                LibraryError::Database(format!("Failed to query custom album cover: {}", e))
            })?;

        Ok(result)
    }

    /// Remove a custom album cover
    pub fn remove_custom_album_cover(&self, album_id: &str) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "DELETE FROM custom_album_covers WHERE album_id = ?1",
                params![album_id],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to remove custom album cover: {}", e))
            })?;
        Ok(())
    }

    /// Get all custom album covers (album_id -> file_path)
    pub fn get_all_custom_album_covers(
        &self,
    ) -> Result<std::collections::HashMap<String, String>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT album_id, custom_image_path FROM custom_album_covers")
            .map_err(|e| LibraryError::Database(format!("Failed to prepare query: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| {
                LibraryError::Database(format!("Failed to query custom album covers: {}", e))
            })?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            if let Ok((album_id, path)) = row {
                map.insert(album_id, path);
            }
        }
        Ok(map)
    }
}
