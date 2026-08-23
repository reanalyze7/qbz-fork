//! The folder-scan engine: cancel token + the blocking-thread scan runner.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use slint::{ComponentHandle, Weak};

use crate::{AppWindow, LibraryScanState};

use super::load::load_folders;
use super::scan_sink::make_sink;

/// Cancel token for the running scan (the port's equivalent of Tauri's
/// `LibraryState.scan_cancel`). `stop_scan` sets it; the core loop checks it
/// at every file boundary.
pub(super) static SCAN_CANCEL: LazyLock<Arc<AtomicBool>> =
    LazyLock::new(|| Arc::new(AtomicBool::new(false)));

/// Run a scan (full when `ids` is None, else the given enabled folders) on a
/// blocking thread, pushing throttled progress to `LibraryScanState`. On
/// finish: reload the folder list, reset the browse models so the tabs
/// re-fetch, and toast the outcome.
pub(super) fn run_scan(weak: Weak<AppWindow>, handle: tokio::runtime::Handle, ids: Option<Vec<i64>>) {
    SCAN_CANCEL.store(false, Ordering::SeqCst);
    let _ = weak.upgrade_in_event_loop(|w| {
        let s = w.global::<LibraryScanState>();
        s.set_scanning(true);
        s.set_scan_status(1);
        s.set_total_files(0);
        s.set_processed_files(0);
        s.set_progress(0.0);
        s.set_current_file("".into());
        s.set_error_count(0);
    });

    let h = handle.clone();
    handle.spawn_blocking(move || {
        let artwork_cache = crate::library_db::artwork_cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let cancel = SCAN_CANCEL.clone();
        let weak_sink = weak.clone();
        let last = Mutex::new(std::time::Instant::now());
        let sink = make_sink(weak_sink, last);

        let ids_ref = ids.as_deref();
        let _ = crate::library_db::with_db(|db| {
            qbz_library::scan_with_progress(db, ids_ref, &artwork_cache, &cancel, &sink)
        });

        // Post-scan: refresh the folder list (last_scan labels) + reset the
        // browse models so the tabs re-fetch the new index on next visit.
        let _ = weak.upgrade_in_event_loop(|w| {
            crate::local_library::reset_browse_models(&w);
        });
        load_folders(weak, h);
    });
}
