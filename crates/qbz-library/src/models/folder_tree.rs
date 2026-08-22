//! Folder-tree view model for the Folders tab

use serde::Serialize;

/// A child entry within a folder of the local-library filesystem
/// hierarchy. Used by the Folders-tab tree view to render one level at
/// a time without preloading the entire tree.
///
/// The `kind` tag is serialised as `snake_case` so the frontend can
/// discriminate via `entry.kind === 'folder' | 'track'`.
///
/// `path` is the absolute filesystem path of the entry. `segment` is
/// the last path component for display. For folder rows,
/// `track_count_under` is the recursive count of `local_tracks`
/// matching `file_path LIKE path || '/%'`. `artwork` is an optional
/// thumbnail path lifted from any track in the subtree (best-effort,
/// not guaranteed to be the album cover).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FolderTreeEntry {
    Folder {
        path: String,
        segment: String,
        track_count_under: u32,
        artwork: Option<String>,
    },
    Track {
        path: String,
        segment: String,
    },
}
