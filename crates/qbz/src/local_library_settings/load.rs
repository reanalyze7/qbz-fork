//! (Re)load the folder list + the per-folder accessibility check.

use std::sync::atomic::Ordering;

use slint::{ComponentHandle, Weak};

use crate::{AppWindow, LibFolderEditState, LibraryFoldersState};

use super::state::{derive, folders_lock, FolderData, FOLDERS, FOLDERS_GEN};

/// (Re)load the folder list. Pure read of the core `get_folders_with_metadata`
/// (the network re-detect + write the Tauri command does lives only in that
/// command, not the core fn — so this has no side effects). Selection is
/// preserved by id across the reload. Network folders get an async
/// accessibility check.
pub fn load_folders(weak: Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let gen = FOLDERS_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<LibraryFoldersState>().set_loading(true);
    });
    let weak2 = weak.clone();
    let check_handle = handle.clone();
    handle.spawn(async move {
        let rows = tokio::task::spawn_blocking(|| {
            crate::library_db::with_db(|db| db.get_folders_with_metadata())
        })
        .await
        .ok()
        .flatten();

        let Some(rows) = rows else {
            let _ = weak2.upgrade_in_event_loop(|w| {
                w.global::<LibraryFoldersState>().set_loading(false);
            });
            crate::toast::error_weak(&weak2, qbz_i18n::t("Couldn't load library folders"));
            return;
        };

        // Preserve selection across reloads.
        let prev_sel: std::collections::HashSet<i64> =
            folders_lock().iter().filter(|f| f.selected).map(|f| f.id).collect();

        let data: Vec<FolderData> = rows
            .into_iter()
            .map(|f| FolderData {
                selected: prev_sel.contains(&f.id),
                accessible: true,
                id: f.id,
                path: f.path,
                alias: f.alias,
                enabled: f.enabled,
                is_network: f.is_network,
                network_fs_type: f.network_fs_type,
                user_override_network: f.user_override_network,
                last_scan: f.last_scan,
            })
            .collect();

        let network: Vec<(i64, String)> = data
            .iter()
            .filter(|f| f.is_network)
            .map(|f| (f.id, f.path.clone()))
            .collect();

        *folders_lock() = data;

        let _ = weak2.upgrade_in_event_loop(move |w| {
            if FOLDERS_GEN.load(Ordering::SeqCst) != gen {
                return;
            }
            w.global::<LibraryFoldersState>().set_loading(false);
            derive(&w);
        });

        for (id, path) in network {
            check_accessible(weak2.clone(), check_handle.clone(), id, path);
        }
    });
}

/// Update one folder's accessibility in the static + UI (and the open modal).
fn update_accessible(weak: &Weak<AppWindow>, id: i64, accessible: bool) {
    {
        let mut g = folders_lock();
        if let Some(f) = g.iter_mut().find(|f| f.id == id) {
            f.accessible = accessible;
        }
    }
    let _ = weak.upgrade_in_event_loop(move |w| {
        derive(&w);
        let es = w.global::<LibFolderEditState>();
        if es.get_open() && es.get_folder_id() as i64 == id {
            es.set_accessible(accessible);
            es.set_checking_accessible(false);
        }
    });
}

/// Check a (network) folder's accessibility. Mirrors Tauri: exists? then
/// read_dir under a 6s timeout; on timeout fall back to exists() so a
/// slow-but-mounted share isn't falsely flagged unavailable.
pub fn check_accessible(
    weak: Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    id: i64,
    path: String,
) {
    handle.spawn(async move {
        if !std::path::Path::new(&path).exists() {
            update_accessible(&weak, id, false);
            return;
        }
        let p = path.clone();
        let res = tokio::time::timeout(
            std::time::Duration::from_secs(6),
            tokio::task::spawn_blocking(move || std::fs::read_dir(&p).is_ok()),
        )
        .await;
        let accessible = match res {
            Ok(Ok(ok)) => ok,
            Ok(Err(_)) => false,
            Err(_) => std::path::Path::new(&path).exists(),
        };
        update_accessible(&weak, id, accessible);
    });
}

