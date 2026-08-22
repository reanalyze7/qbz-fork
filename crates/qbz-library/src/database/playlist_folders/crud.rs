//! Create, list, and fetch playlist folders.

use rusqlite::{params, OptionalExtension};

use crate::database::PlaylistFolder;
use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    // === Playlist Folders ===

    /// Create a new playlist folder
    pub fn create_playlist_folder(
        &self,
        name: &str,
        icon_type: Option<&str>,
        icon_preset: Option<&str>,
        icon_color: Option<&str>,
    ) -> Result<PlaylistFolder, LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let id = uuid::Uuid::new_v4().to_string();

        // Get the next position
        let max_position: i32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(position), -1) FROM playlist_folders",
                [],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        let folder = PlaylistFolder {
            id: id.clone(),
            name: name.to_string(),
            icon_type: icon_type.unwrap_or("preset").to_string(),
            icon_preset: icon_preset.unwrap_or("folder").to_string(),
            icon_color: icon_color.unwrap_or("#6366f1").to_string(),
            custom_image_path: None,
            is_hidden: false,
            position: max_position + 1,
            created_at: now,
            updated_at: now,
        };

        self.conn.execute(
            "INSERT INTO playlist_folders (id, name, icon_type, icon_preset, icon_color, custom_image_path, is_hidden, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                &folder.id,
                &folder.name,
                &folder.icon_type,
                &folder.icon_preset,
                &folder.icon_color,
                &folder.custom_image_path,
                folder.is_hidden as i32,
                folder.position,
                folder.created_at,
                folder.updated_at,
            ],
        ).map_err(|e| LibraryError::Database(format!("Failed to create playlist folder: {}", e)))?;

        Ok(folder)
    }

    /// Get all playlist folders
    pub fn get_all_playlist_folders(&self) -> Result<Vec<PlaylistFolder>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, icon_type, icon_preset, icon_color, custom_image_path, is_hidden, position, created_at, updated_at
             FROM playlist_folders ORDER BY position ASC"
        ).map_err(|e| LibraryError::Database(format!("Failed to prepare statement: {}", e)))?;

        let folders = stmt
            .query_map([], |row| {
                Ok(PlaylistFolder {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    icon_type: row.get(2)?,
                    icon_preset: row.get(3)?,
                    icon_color: row.get(4)?,
                    custom_image_path: row.get(5)?,
                    is_hidden: row.get::<_, i32>(6)? != 0,
                    position: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(|e| {
                LibraryError::Database(format!("Failed to query playlist folders: {}", e))
            })?;

        folders.collect::<Result<Vec<_>, _>>().map_err(|e| {
            LibraryError::Database(format!("Failed to collect playlist folders: {}", e))
        })
    }

    /// Get a playlist folder by ID
    pub fn get_playlist_folder(
        &self,
        folder_id: &str,
    ) -> Result<Option<PlaylistFolder>, LibraryError> {
        let result = self.conn.query_row(
            "SELECT id, name, icon_type, icon_preset, icon_color, custom_image_path, is_hidden, position, created_at, updated_at
             FROM playlist_folders WHERE id = ?1",
            params![folder_id],
            |row| {
                Ok(PlaylistFolder {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    icon_type: row.get(2)?,
                    icon_preset: row.get(3)?,
                    icon_color: row.get(4)?,
                    custom_image_path: row.get(5)?,
                    is_hidden: row.get::<_, i32>(6)? != 0,
                    position: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        ).optional()
        .map_err(|e| LibraryError::Database(format!("Failed to get playlist folder: {}", e)))?;

        Ok(result)
    }
}
