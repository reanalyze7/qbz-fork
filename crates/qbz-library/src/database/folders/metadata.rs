use rusqlite::{params, OptionalExtension};

use crate::LibraryError;

use super::super::LibraryDatabase;
use super::super::LibraryFolder;

impl LibraryDatabase {
    /// Get all library folders with full metadata
    pub fn get_folders_with_metadata(&self) -> Result<Vec<LibraryFolder>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, path, alias, enabled, is_network, network_fs_type, user_override_network, last_scan
                 FROM library_folders ORDER BY path"
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(LibraryFolder {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    alias: row.get(2)?,
                    enabled: row.get::<_, i32>(3)? != 0,
                    is_network: row.get::<_, i32>(4).unwrap_or(0) != 0,
                    network_fs_type: row.get(5)?,
                    user_override_network: row.get::<_, i32>(6).unwrap_or(0) != 0,
                    last_scan: row.get(7)?,
                })
            })
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut folders = Vec::new();
        for folder in rows {
            folders.push(folder.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(folders)
    }

    /// Get a single folder by ID
    pub fn get_folder_by_id(&self, id: i64) -> Result<Option<LibraryFolder>, LibraryError> {
        let result = self
            .conn
            .query_row(
                "SELECT id, path, alias, enabled, is_network, network_fs_type, user_override_network, last_scan
                 FROM library_folders WHERE id = ?",
                params![id],
                |row| {
                    Ok(LibraryFolder {
                        id: row.get(0)?,
                        path: row.get(1)?,
                        alias: row.get(2)?,
                        enabled: row.get::<_, i32>(3)? != 0,
                        is_network: row.get::<_, i32>(4).unwrap_or(0) != 0,
                        network_fs_type: row.get(5)?,
                        user_override_network: row.get::<_, i32>(6).unwrap_or(0) != 0,
                        last_scan: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        Ok(result)
    }
}
