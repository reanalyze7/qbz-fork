//! Blocking fetch of one folder's direct tracks + resolved-cover subfolders.

use crate::local_library::cover::find_folder_cover;

pub(crate) type DetailFetch = (Vec<qbz_library::LocalTrack>, Vec<qbz_library::FolderTreeEntry>);

/// Fetch a folder's direct tracks and immediate subfolders, resolving an
/// on-disk cover for any subfolder whose indexed `artwork_path` is empty.
/// Blocking — must run off the UI thread.
pub(crate) fn fetch_folder_detail(path: &str) -> DetailFetch {
    let tracks =
        crate::library_db::with_db(|db| db.list_folder_tracks(path, false)).unwrap_or_default();
    // Resolve a real on-disk cover for each subfolder whose indexed
    // artwork_path is empty (no embedded art / never backfilled) — the
    // image can sit under any of a dozen names (cover/folder/front/art/
    // <album>.jpg, …). Off-thread, so the fs scan is fine here.
    let children = crate::library_db::with_db(|db| db.list_folder_children(path, false))
        .unwrap_or_default()
        .into_iter()
        .map(|e| match e {
            qbz_library::FolderTreeEntry::Folder {
                path,
                segment,
                track_count_under,
                artwork,
            } => {
                let artwork = artwork
                    .filter(|a| !a.is_empty())
                    .or_else(|| find_folder_cover(std::path::Path::new(&path)));
                qbz_library::FolderTreeEntry::Folder {
                    path,
                    segment,
                    track_count_under,
                    artwork,
                }
            }
            other => other,
        })
        .collect::<Vec<_>>();
    (tracks, children)
}
