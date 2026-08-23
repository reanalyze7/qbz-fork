//! The three scan entry points: scan everything, scan one folder, cancel.

use std::sync::atomic::Ordering;

use slint::{ComponentHandle, Weak};

use crate::{AppWindow, LibFolderEditState};

use super::scan::{run_scan, SCAN_CANCEL};
use super::state::folders_lock;

/// Scan every enabled folder. Guards on an empty list.
pub fn scan_all(weak: Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let empty = folders_lock().is_empty();
    if empty {
        crate::toast::error_weak(&weak, qbz_i18n::t("Add a folder before scanning"));
        return;
    }
    run_scan(weak, handle, None);
}

/// Scan a single folder (from the settings modal). Closes the modal first.
pub fn scan_folder(weak: Weak<AppWindow>, handle: tokio::runtime::Handle, id: i64) {
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<LibFolderEditState>().set_open(false);
    });
    run_scan(weak, handle, Some(vec![id]));
}

/// Request cancellation of the running scan.
pub fn stop_scan() {
    SCAN_CANCEL.store(true, Ordering::SeqCst);
}
