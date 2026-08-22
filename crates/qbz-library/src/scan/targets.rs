use std::path::Path;

use crate::{LibraryDatabase, LibraryError, LibraryFolder};

/// Resolve the set of folders a scan should cover (`folder_ids = None` means
/// every enabled folder; `Some(ids)` means only those enabled folders), and
/// refresh network-mount detection for each of them. Returns the resolved
/// targets plus `single` (whether this is a folder-subset scan).
pub(super) fn resolve_targets(
    db: &LibraryDatabase,
    folder_ids: Option<&[i64]>,
) -> Result<(Vec<LibraryFolder>, bool), LibraryError> {
    let all = db.get_folders_with_metadata()?;
    let targets: Vec<LibraryFolder> = match folder_ids {
        None => all.into_iter().filter(|f| f.enabled).collect(),
        Some(ids) => all
            .into_iter()
            .filter(|f| f.enabled && ids.contains(&f.id))
            .collect(),
    };
    if targets.is_empty() {
        return Err(LibraryError::Other("No library folders to scan".to_string()));
    }
    let single = folder_ids.is_some();

    refresh_network_status(db, &targets);

    Ok((targets, single))
}

/// Refresh network detection for non-overridden folders being scanned.
fn refresh_network_status(db: &LibraryDatabase, targets: &[LibraryFolder]) {
    for f in targets {
        if f.user_override_network {
            continue;
        }
        let p = Path::new(&f.path);
        let is_net = crate::mount_info::is_network_path(p);
        if is_net != f.is_network {
            let fs = if is_net {
                crate::mount_info::network_fs_label(p)
            } else {
                None
            };
            let _ = db.update_folder_settings(
                f.id,
                f.alias.as_deref(),
                f.enabled,
                is_net,
                fs.as_deref(),
                false,
            );
        }
    }
}
