use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{LibraryDatabase, ScanError, ScanStatus};

use super::cue::process_cue_file;
use super::event::ScanEvent;
use super::outcome::ScanOutcome;

/// Process a folder's CUE files (one file -> several virtual tracks),
/// checking `cancel` at every file boundary.
pub(super) fn process_cue_files(
    db: &LibraryDatabase,
    cue_files: &[PathBuf],
    artwork_cache: &std::path::Path,
    cancel: &AtomicBool,
    on_event: &(dyn Fn(ScanEvent) + Send + Sync),
    all_errors: &mut Vec<ScanError>,
    total: u32,
    processed: &mut u32,
) -> ScanOutcome {
    for cue_path in cue_files {
        if cancel.load(Ordering::Relaxed) {
            on_event(ScanEvent::Finished {
                status: ScanStatus::Cancelled,
                errors: std::mem::take(all_errors),
            });
            return ScanOutcome::Cancelled;
        }
        on_event(ScanEvent::FileStarted {
            path: cue_path.to_string_lossy().to_string(),
        });
        if let Err(e) = process_cue_file(db, cue_path, artwork_cache) {
            all_errors.push(ScanError {
                file_path: cue_path.to_string_lossy().to_string(),
                error: e,
            });
        }
        *processed += 1;
        on_event(ScanEvent::FileDone {
            processed: *processed,
            total,
        });
    }
    ScanOutcome::Continue
}
