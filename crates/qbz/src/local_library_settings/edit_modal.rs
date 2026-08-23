//! The folder-settings modal: open + persist.

use slint::{ComponentHandle, Weak};

use crate::{AppWindow, LibFolderEditState};

use super::load::{check_accessible, load_folders};
use super::state::{fs_label_to_index, folders_lock, last_scan_label, FolderData};

/// Open the folder-settings modal for `id` (or the single selected folder
/// when `id == 0`, as the toolbar Edit button passes).
pub fn edit_folder(weak: Weak<AppWindow>, handle: tokio::runtime::Handle, id: i64) {
    let f = {
        let g = folders_lock();
        if id > 0 {
            g.iter().find(|f| f.id == id).cloned()
        } else {
            let sel: Vec<FolderData> = g.iter().filter(|f| f.selected).cloned().collect();
            if sel.len() == 1 {
                Some(sel[0].clone())
            } else {
                None
            }
        }
    };
    let Some(f) = f else {
        return;
    };
    let is_network = f.is_network;
    let fid = f.id;
    let path = f.path.clone();
    let _ = weak.upgrade_in_event_loop(move |w| {
        let es = w.global::<LibFolderEditState>();
        es.set_folder_id(f.id as i32);
        es.set_path(f.path.clone().into());
        es.set_alias(f.alias.clone().unwrap_or_default().into());
        es.set_enabled(f.enabled);
        es.set_is_network(f.is_network);
        es.set_user_override_network(f.user_override_network);
        es.set_fs_type_index(fs_label_to_index(f.network_fs_type.as_deref()));
        es.set_accessible(f.accessible);
        es.set_checking_accessible(f.is_network);
        es.set_last_scan_label(last_scan_label(f.last_scan).into());
        es.set_open(true);
    });
    if is_network {
        check_accessible(weak, handle, fid, path);
    }
}

/// Persist folder settings from the modal. fs-type "auto" re-detects (network
/// only); a non-auto label is stored verbatim.
#[allow(clippy::too_many_arguments)]
pub fn save_folder_settings(
    weak: Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    id: i64,
    alias: String,
    enabled: bool,
    is_network: bool,
    fs_type: String,
    user_override: bool,
) {
    let path = folders_lock()
        .iter()
        .find(|f| f.id == id)
        .map(|f| f.path.clone())
        .unwrap_or_default();
    let h = handle.clone();
    handle.spawn(async move {
        let ok = tokio::task::spawn_blocking(move || {
            let fs_opt: Option<String> = if !is_network {
                None
            } else if fs_type == "auto" {
                qbz_library::network_fs_label(std::path::Path::new(&path))
            } else {
                Some(fs_type)
            };
            let alias_opt = if alias.trim().is_empty() {
                None
            } else {
                Some(alias)
            };
            crate::library_db::with_db(|db| {
                db.update_folder_settings(
                    id,
                    alias_opt.as_deref(),
                    enabled,
                    is_network,
                    fs_opt.as_deref(),
                    user_override,
                )
            })
            .is_some()
        })
        .await
        .unwrap_or(false);

        if !ok {
            crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't save folder settings"));
            return;
        }
        let _ = weak.upgrade_in_event_loop(|w| {
            w.global::<LibFolderEditState>().set_open(false);
        });
        load_folders(weak, h);
    });
}
