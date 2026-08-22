//! Partial-update of an existing playlist folder's metadata.

use rusqlite::params;

use crate::database::PlaylistFolder;
use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Update a playlist folder
    pub fn update_playlist_folder(
        &self,
        folder_id: &str,
        name: Option<&str>,
        icon_type: Option<&str>,
        icon_preset: Option<&str>,
        icon_color: Option<&str>,
        custom_image_path: Option<Option<&str>>,
        is_hidden: Option<bool>,
    ) -> Result<PlaylistFolder, LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Get existing folder
        let existing = self
            .get_playlist_folder(folder_id)?
            .ok_or_else(|| LibraryError::Database("Folder not found".to_string()))?;

        let new_name = name.unwrap_or(&existing.name);
        let new_icon_type = icon_type.unwrap_or(&existing.icon_type);
        let new_icon_preset = icon_preset.unwrap_or(&existing.icon_preset);
        let new_icon_color = icon_color.unwrap_or(&existing.icon_color);
        let new_custom_image_path =
            custom_image_path.unwrap_or(existing.custom_image_path.as_deref());
        let new_is_hidden = is_hidden.unwrap_or(existing.is_hidden);

        self.conn.execute(
            "UPDATE playlist_folders SET name = ?1, icon_type = ?2, icon_preset = ?3, icon_color = ?4,
             custom_image_path = ?5, is_hidden = ?6, updated_at = ?7 WHERE id = ?8",
            params![
                new_name,
                new_icon_type,
                new_icon_preset,
                new_icon_color,
                new_custom_image_path,
                new_is_hidden as i32,
                now,
                folder_id,
            ],
        ).map_err(|e| LibraryError::Database(format!("Failed to update playlist folder: {}", e)))?;

        self.get_playlist_folder(folder_id)?
            .ok_or_else(|| LibraryError::Database("Folder not found after update".to_string()))
    }
}
