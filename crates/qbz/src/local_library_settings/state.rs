//! The authoritative folder set + its derived-model plumbing.
use slint::ComponentHandle;

use std::sync::atomic::AtomicU64;
use std::sync::{LazyLock, Mutex};

use slint::{ModelRc, VecModel};

use crate::{AppWindow, LibraryFolderItem, LibraryFoldersState};

/// One registered folder + its UI selection state. The authoritative copy;
/// the Slint model is derived (filtered) from this.
#[derive(Clone)]
pub(super) struct FolderData {
    pub(super) id: i64,
    pub(super) path: String,
    pub(super) alias: Option<String>,
    pub(super) enabled: bool,
    pub(super) is_network: bool,
    pub(super) network_fs_type: Option<String>,
    pub(super) user_override_network: bool,
    pub(super) last_scan: Option<i64>,
    pub(super) accessible: bool,
    pub(super) selected: bool,
}

pub(super) static FOLDERS: LazyLock<Mutex<Vec<FolderData>>> = LazyLock::new(|| Mutex::new(Vec::new()));
/// Bumped on every (re)load so a stale in-flight load is dropped on apply.
pub(super) static FOLDERS_GEN: AtomicU64 = AtomicU64::new(0);

pub(super) fn folders_lock() -> std::sync::MutexGuard<'static, Vec<FolderData>> {
    FOLDERS.lock().unwrap_or_else(|e| e.into_inner())
}

pub(super) fn display_name(f: &FolderData) -> String {
    match f.alias.as_deref() {
        Some(a) if !a.is_empty() => a.to_string(),
        _ => f.path.clone(),
    }
}

/// Format a folder's `last_scan` (unix seconds, 0/None = never) for display.
pub(super) fn last_scan_label(ts: Option<i64>) -> String {
    match ts {
        None | Some(0) => qbz_i18n::t("Never"),
        Some(secs) => chrono::DateTime::from_timestamp(secs, 0)
            .map(|dt| {
                dt.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_else(|| qbz_i18n::t("Never")),
    }
}

/// fs-type label -> the modal's QbzSelect index (0=auto,1 cifs … 8 other).
pub(super) fn fs_label_to_index(label: Option<&str>) -> i32 {
    match label.unwrap_or("") {
        "cifs" => 1,
        "nfs" => 2,
        "sshfs" => 3,
        "rclone" => 4,
        "webdav" => 5,
        "glusterfs" => 6,
        "ceph" => 7,
        "other" => 8,
        _ => 0,
    }
}

pub(super) fn to_item(f: &FolderData) -> LibraryFolderItem {
    LibraryFolderItem {
        id: f.id as i32,
        path: f.path.clone().into(),
        alias: f.alias.clone().unwrap_or_default().into(),
        display_name: display_name(f).into(),
        enabled: f.enabled,
        is_network: f.is_network,
        network_fs_type: f.network_fs_type.clone().unwrap_or_default().into(),
        user_override_network: f.user_override_network,
        last_scan: f.last_scan.unwrap_or(0) as i32,
        last_scan_label: last_scan_label(f.last_scan).into(),
        accessible: f.accessible,
        selected: f.selected,
    }
}

/// Derive the filtered render model + selected-count from the static set.
pub(super) fn derive(window: &AppWindow) {
    let s = window.global::<LibraryFoldersState>();
    let filter = s.get_filter().to_lowercase();
    let q = filter.trim();
    let guard = folders_lock();
    let items: Vec<LibraryFolderItem> = guard
        .iter()
        .filter(|f| {
            q.is_empty()
                || display_name(f).to_lowercase().contains(q)
                || f.path.to_lowercase().contains(q)
        })
        .map(to_item)
        .collect();
    let selected = guard.iter().filter(|f| f.selected).count() as i32;
    drop(guard);
    s.set_folders(ModelRc::new(VecModel::from(items)));
    s.set_selected_count(selected);
}
