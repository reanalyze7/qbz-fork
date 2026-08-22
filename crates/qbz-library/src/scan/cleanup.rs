use std::path::Path;

use crate::{LibraryDatabase, LibraryFolder};

use super::helpers::now_secs;

fn folder_prefix(path: &str) -> String {
    if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{}/", path)
    }
}

/// Remove tracks whose files no longer exist. Full scan checks the whole DB;
/// single-folder scan only the scanned folders' subtrees.
///
/// GUARD: a network folder whose mount is currently DOWN must not have its
/// subtree treated as missing — while unmounted every stat fails, and
/// deleting the rows would wipe a whole folder's index over a transient
/// condition (e.g. a reboot where the share didn't auto-mount). Those
/// subtrees are skipped; they rehabilitate on the next scan after remount.
pub(super) fn cleanup_missing(db: &LibraryDatabase, targets: &[LibraryFolder], single: bool) {
    // Re-fetched intentionally: folder metadata may have changed during the
    // scan via the network-refresh loop.
    let unavailable_prefixes: Vec<String> = db
        .get_folders_with_metadata()
        .map(|folders| {
            folders
                .iter()
                .filter(|f| f.is_network && std::fs::read_dir(&f.path).is_err())
                .map(|f| folder_prefix(&f.path))
                .collect()
        })
        .unwrap_or_default();
    let under_unavailable =
        |p: &str| unavailable_prefixes.iter().any(|pre| p.starts_with(pre.as_str()));

    if let Ok(tracks) = db.get_all_track_paths() {
        let missing: Vec<i64> = if single {
            let prefixes: Vec<String> = targets.iter().map(|f| folder_prefix(&f.path)).collect();
            tracks
                .iter()
                .filter(|(_, p)| {
                    prefixes.iter().any(|pre| p.starts_with(pre))
                        && !under_unavailable(p)
                        && !Path::new(p).exists()
                })
                .map(|(id, _)| *id)
                .collect()
        } else {
            tracks
                .iter()
                .filter(|(_, p)| !under_unavailable(p) && !Path::new(p).exists())
                .map(|(id, _)| *id)
                .collect()
        };
        for chunk in missing.chunks(500) {
            let _ = db.delete_tracks_by_ids(chunk);
        }
    }
}

/// Stamp each scanned folder's last_scan time (improvement: full scan too).
pub(super) fn stamp_last_scan(db: &LibraryDatabase, targets: &[LibraryFolder]) {
    let now = now_secs();
    for f in targets {
        let _ = db.update_folder_scan_time(&f.path, now);
    }
}
