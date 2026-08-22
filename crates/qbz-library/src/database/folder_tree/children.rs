use rusqlite::params;

use crate::{FolderTreeEntry, LibraryError};

use super::super::helpers::escape_like_pattern;
use super::super::LibraryDatabase;
use super::children_query::children_query;
use super::children_sort::sort_tree_entries;

impl LibraryDatabase {
    /// List the immediate children of a folder in the local-library
    /// filesystem hierarchy.
    ///
    /// Walks `local_tracks.file_path` and computes one row per direct
    /// child. Returns folders first (alphabetical, case-insensitive),
    /// then tracks (alphabetical, case-insensitive).
    ///
    /// Filters `COALESCE(source, 'user') = 'user'` so Qobuz offline
    /// downloads are excluded.
    ///
    /// `parent_path` is the absolute path of the folder whose children
    /// to enumerate. The `_` and `%` characters are escaped before
    /// binding to defend against pattern-injection on paths that
    /// contain SQL LIKE metacharacters.
    pub fn list_folder_children(
        &self,
        parent_path: &str,
        exclude_network_folders: bool,
    ) -> Result<Vec<FolderTreeEntry>, LibraryError> {
        let escaped_prefix = escape_like_pattern(parent_path);

        // Network folder filter: exclude tracks whose file_path starts
        // with any registered network-mount folder path. Mirrors the
        // mechanism used by `get_albums_with_full_filter` so tree rail
        // visibility matches flat-mode + recursive playback.
        let network_filter = if exclude_network_folders {
            "AND NOT EXISTS ( \
                SELECT 1 FROM library_folders nf \
                WHERE nf.is_network = 1 \
                AND local_tracks.file_path LIKE nf.path || '%' \
            )"
        } else {
            ""
        };

        let sql = children_query(network_filter);

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        // ?1 bound with the unescaped path (used in length() arithmetic
        // on the row's stored file_path; that storage is unescaped).
        // ?2 bound with the LIKE-escaped pattern prefix.
        let rows = stmt
            .query_map(params![parent_path, escaped_prefix], |row| {
                let segment: String = row.get(0)?;
                let kind: String = row.get(1)?;
                let count: u32 = row.get(2)?;
                let artwork: Option<String> = row.get(3)?;
                let one_file_path: Option<String> = row.get(4)?;
                Ok((segment, kind, count, artwork, one_file_path))
            })
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut entries: Vec<FolderTreeEntry> = Vec::new();
        for row in rows {
            let (segment, kind, count, artwork, one_file_path) =
                row.map_err(|e| LibraryError::Database(e.to_string()))?;
            match kind.as_str() {
                "folder" => {
                    let path = format!("{}/{}", parent_path, segment);
                    entries.push(FolderTreeEntry::Folder {
                        path,
                        segment,
                        track_count_under: count,
                        artwork,
                    });
                }
                "track" => {
                    // Use the actual file_path so paths with edge-case
                    // characters round-trip exactly as stored.
                    let path = one_file_path
                        .unwrap_or_else(|| format!("{}/{}", parent_path, segment));
                    entries.push(FolderTreeEntry::Track { path, segment });
                }
                _ => {
                    // Unknown kind — skip defensively.
                }
            }
        }

        sort_tree_entries(&mut entries);

        Ok(entries)
    }
}
