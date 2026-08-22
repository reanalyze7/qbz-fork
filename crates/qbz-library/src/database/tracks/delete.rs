use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Delete all tracks in a folder
    pub fn delete_tracks_in_folder(&self, folder: &str) -> Result<usize, LibraryError> {
        let pattern = format!("{}%", folder);
        let count = self
            .conn
            .execute(
                "DELETE FROM local_tracks WHERE file_path LIKE ?",
                params![pattern],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(count)
    }

    /// Delete all tracks under a folder, matching a path prefix terminated by
    /// the separator so a sibling like `/music/jazz2` is NOT removed when
    /// deleting `/music/jazz`. Use this for folder removal — the older
    /// `delete_tracks_in_folder` (kept for backward behavior compatibility with
    /// the Tauri command) has a prefix-collision bug (`{}%`, no separator).
    pub fn delete_tracks_in_folder_prefixed(&self, folder: &str) -> Result<usize, LibraryError> {
        let pattern = format!("{}/%", folder.trim_end_matches('/'));
        let count = self
            .conn
            .execute(
                "DELETE FROM local_tracks WHERE file_path LIKE ?",
                params![pattern],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(count)
    }

    /// Distinct `album_group_key`s of the indexed tracks under `folder` — the
    /// same keys used as the playback/Recently-Played album id. Call BEFORE
    /// deleting the folder so the frontend can prune those albums from the
    /// recently-played store.
    pub fn album_keys_in_folder(&self, folder: &str) -> Result<Vec<String>, LibraryError> {
        let pattern = format!("{}/%", folder.trim_end_matches('/'));
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT album_group_key FROM local_tracks
                 WHERE file_path LIKE ? AND album_group_key IS NOT NULL AND album_group_key != ''",
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        let rows = stmt
            .query_map(params![pattern], |row| row.get::<_, String>(0))
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        let mut keys = Vec::new();
        for k in rows {
            keys.push(k.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(keys)
    }

    /// Remove a folder and its indexed tracks (separator-safe cascade). Mirrors
    /// the Tauri remove-folder command order: drop the folder row, then the
    /// tracks under it. Returns the number of tracks removed.
    pub fn remove_folder_with_tracks(&self, path: &str) -> Result<usize, LibraryError> {
        self.remove_folder(path)?;
        self.delete_tracks_in_folder_prefixed(path)
    }

    /// Clear all LOCAL library tracks (preserves Qobuz downloads)
    pub fn clear_all_tracks(&self) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "DELETE FROM local_tracks WHERE source IS NULL OR source != 'qobuz_download'",
                [],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(())
    }

    /// Delete tracks by their IDs
    pub fn delete_tracks_by_ids(&self, ids: &[i64]) -> Result<usize, LibraryError> {
        if ids.is_empty() {
            return Ok(0);
        }

        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "DELETE FROM local_tracks WHERE id IN ({})",
            placeholders.join(",")
        );

        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();

        let count = self
            .conn
            .execute(&query, params.as_slice())
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        Ok(count)
    }
}
