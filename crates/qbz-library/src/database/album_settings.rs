//! Album settings: per-album "hidden" flag persisted independently of
//! scanned metadata, so a user can hide an album from the library view
//! without touching the underlying files.

use rusqlite::{params, OptionalExtension};

use crate::LibraryError;

use super::LibraryDatabase;

impl LibraryDatabase {
    /// Get album settings
    pub fn get_album_settings(
        &self,
        album_group_key: &str,
    ) -> Result<Option<crate::AlbumSettings>, LibraryError> {
        let result = self
            .conn
            .query_row(
                "SELECT album_group_key, hidden, created_at, updated_at
             FROM album_settings WHERE album_group_key = ?1",
                params![album_group_key],
                |row| {
                    Ok(crate::AlbumSettings {
                        album_group_key: row.get(0)?,
                        hidden: row.get::<_, i32>(1)? != 0,
                        created_at: row.get(2)?,
                        updated_at: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|e| LibraryError::Database(format!("Failed to get album settings: {}", e)))?;

        Ok(result)
    }

    /// Set album hidden status
    pub fn set_album_hidden(
        &self,
        album_group_key: &str,
        hidden: bool,
    ) -> Result<(), LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        self.conn
            .execute(
                "INSERT INTO album_settings (album_group_key, hidden, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(album_group_key) DO UPDATE SET
                hidden = excluded.hidden,
                updated_at = excluded.updated_at",
                params![album_group_key, hidden as i32, now, now],
            )
            .map_err(|e| LibraryError::Database(format!("Failed to set album hidden: {}", e)))?;

        Ok(())
    }

    /// Get all hidden albums
    pub fn get_hidden_albums(&self) -> Result<Vec<String>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT album_group_key FROM album_settings WHERE hidden = 1")
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| LibraryError::Database(e.to_string()))
    }
}
