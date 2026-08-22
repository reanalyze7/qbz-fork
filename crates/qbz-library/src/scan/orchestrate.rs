use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use crate::{LibraryDatabase, LibraryError, LibraryScanner, ScanError, ScanStatus};

use super::cleanup::{cleanup_missing, stamp_last_scan};
use super::event::ScanEvent;
use super::folder_loop::scan_folder;
use super::outcome::ScanOutcome;
use super::targets::resolve_targets;

/// Scan the library (or a single folder set) with progress + cancellation.
///
/// `folder_ids = None` scans every ENABLED folder (full scan); `Some(&[id])`
/// scans only those enabled folders (single/per-folder parity). `artwork_cache`
/// is the cache dir for extracted/copied covers. `cancel` is checked at every
/// file boundary; on cancel the scan returns early with `Finished{Cancelled}`
/// and does NOT run cleanup (Tauri parity). `on_event` receives each step.
///
/// Improvements over the Tauri loop (both verified against source):
/// - the full scan updates each folder's `last_scan` on success (Tauri only
///   did this for single-folder scans);
/// - network status is re-detected for every scanned folder that is not
///   user-overridden (Tauri only did this for single-folder scans).
pub fn scan_with_progress(
    db: &LibraryDatabase,
    folder_ids: Option<&[i64]>,
    artwork_cache: &Path,
    cancel: &AtomicBool,
    on_event: &(dyn Fn(ScanEvent) + Send + Sync),
) -> Result<(), LibraryError> {
    let (targets, single) = resolve_targets(db, folder_ids)?;

    on_event(ScanEvent::Started);

    let scanner = LibraryScanner::new();
    let mut all_errors: Vec<ScanError> = Vec::new();
    let mut sidecar_cache = HashMap::new();
    let mut total: u32 = 0;
    let mut processed: u32 = 0;

    for folder in &targets {
        let outcome = scan_folder(
            db,
            folder,
            &scanner,
            artwork_cache,
            cancel,
            on_event,
            &mut all_errors,
            &mut sidecar_cache,
            &mut total,
            &mut processed,
        );
        if let ScanOutcome::Cancelled = outcome {
            return Ok(());
        }
    }

    on_event(ScanEvent::Cleanup);
    cleanup_missing(db, &targets, single);
    stamp_last_scan(db, &targets);

    on_event(ScanEvent::Finished {
        status: ScanStatus::Complete,
        errors: all_errors,
    });
    Ok(())
}
