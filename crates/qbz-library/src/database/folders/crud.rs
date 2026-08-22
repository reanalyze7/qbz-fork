use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Add a folder to the library with optional network info
    pub fn add_folder(&self, path: &str) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO library_folders (path) VALUES (?)",
                params![path],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(())
    }

    /// Add a folder with network detection info
    pub fn add_folder_with_network_info(
        &self,
        path: &str,
        is_network: bool,
        network_fs_type: Option<&str>,
    ) -> Result<i64, LibraryError> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO library_folders (path, is_network, network_fs_type) VALUES (?, ?, ?)",
                params![path, is_network as i32, network_fs_type],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        // Get the folder ID (either newly inserted or existing)
        let id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM library_folders WHERE path = ?",
                params![path],
                |row| row.get(0),
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        Ok(id)
    }

    /// Remove a folder from the library
    pub fn remove_folder(&self, path: &str) -> Result<(), LibraryError> {
        self.conn
            .execute("DELETE FROM library_folders WHERE path = ?", params![path])
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(())
    }

    /// Get all enabled library folders (paths only, for scanning)
    pub fn get_folders(&self) -> Result<Vec<String>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT path FROM library_folders WHERE enabled = 1")
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut folders = Vec::new();
        for path in rows {
            folders.push(path.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(folders)
    }

    /// Get paths of all network folders (for offline filtering)
    pub fn get_network_folder_paths(&self) -> Result<Vec<String>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT path FROM library_folders WHERE is_network = 1")
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut folders = Vec::new();
        for path in rows {
            folders.push(path.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(folders)
    }
}
