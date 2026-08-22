use rusqlite::{params, OptionalExtension};

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Update folder settings
    pub fn update_folder_settings(
        &self,
        id: i64,
        alias: Option<&str>,
        enabled: bool,
        is_network: bool,
        network_fs_type: Option<&str>,
        user_override_network: bool,
    ) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "UPDATE library_folders
                 SET alias = ?, enabled = ?, is_network = ?, network_fs_type = ?, user_override_network = ?
                 WHERE id = ?",
                params![alias, enabled as i32, is_network as i32, network_fs_type, user_override_network as i32, id],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(())
    }

    /// Set folder enabled state
    pub fn set_folder_enabled(&self, id: i64, enabled: bool) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "UPDATE library_folders SET enabled = ? WHERE id = ?",
                params![enabled as i32, id],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(())
    }

    /// Update last scan time for a folder
    pub fn update_folder_scan_time(&self, path: &str, timestamp: i64) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "UPDATE library_folders SET last_scan = ? WHERE path = ?",
                params![timestamp, path],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(())
    }

    /// Update folder path (moves the folder to a new location)
    /// This also clears the last_scan since the new path needs to be scanned
    pub fn update_folder_path(&self, id: i64, new_path: &str) -> Result<(), LibraryError> {
        // Check if new path already exists as a different folder
        let existing: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM library_folders WHERE path = ? AND id != ?",
                params![new_path, id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        if existing.is_some() {
            return Err(LibraryError::Database(
                "A folder with this path already exists".to_string(),
            ));
        }

        self.conn
            .execute(
                "UPDATE library_folders SET path = ?, last_scan = NULL WHERE id = ?",
                params![new_path, id],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(())
    }
}
