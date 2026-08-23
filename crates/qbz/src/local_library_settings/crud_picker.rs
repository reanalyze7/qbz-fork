//! Picker-driven folder mutations: add a folder, change an existing one's
//! path.

use slint::{ComponentHandle, Weak};

use crate::{AppWindow, LibFolderEditState};

use super::load::load_folders;

/// Add a folder via the native directory picker, auto-detecting network type.
pub fn add_folder(weak: Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let h = handle.clone();
    handle.spawn(async move {
        let Some(dir) = rfd::AsyncFileDialog::new()
            .set_title(&qbz_i18n::t("Select music folder"))
            .pick_folder()
            .await
        else {
            return;
        };
        let path = dir.path().to_string_lossy().to_string();
        let p = path.clone();
        let (added, is_net) = tokio::task::spawn_blocking(move || {
            let pb = std::path::Path::new(&p);
            let is_net = qbz_library::is_network_path(pb);
            let fs = if is_net {
                qbz_library::network_fs_label(pb)
            } else {
                None
            };
            let ok = crate::library_db::with_db(|db| {
                db.add_folder_with_network_info(&p, is_net, fs.as_deref())
            })
            .is_some();
            (ok, is_net)
        })
        .await
        .unwrap_or((false, false));

        if !added {
            crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't add folder"));
            return;
        }
        if is_net {
            crate::toast::success_weak(&weak, qbz_i18n::t("Network folder detected"));
        }
        load_folders(weak, h);
    });
}

/// Change a folder's path via the picker (resets its last_scan; rejects dups).
pub fn change_folder_path(weak: Weak<AppWindow>, handle: tokio::runtime::Handle, id: i64) {
    let h = handle.clone();
    handle.spawn(async move {
        let Some(dir) = rfd::AsyncFileDialog::new()
            .set_title(&qbz_i18n::t("Select music folder"))
            .pick_folder()
            .await
        else {
            return;
        };
        let new_path = dir.path().to_string_lossy().to_string();
        let np = new_path.clone();
        let ok = tokio::task::spawn_blocking(move || {
            crate::library_db::with_db(|db| db.update_folder_path(id, &np)).is_some()
        })
        .await
        .unwrap_or(false);

        if !ok {
            crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't change folder location (path may already exist)"));
            return;
        }
        let np2 = new_path.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            let es = w.global::<LibFolderEditState>();
            if es.get_folder_id() as i64 == id {
                es.set_path(np2.into());
                es.set_last_scan_label(qbz_i18n::t("Never").into());
            }
        });
        load_folders(weak, h);
    });
}
