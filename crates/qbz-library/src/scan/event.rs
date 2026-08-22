use crate::{ScanError, ScanStatus};

/// One step of a scan, pushed to the caller. The caller maps these onto its
/// own progress surface (and may coalesce the per-file stream).
pub enum ScanEvent {
    /// Scan started (status = Scanning, counters reset).
    Started,
    /// A folder's file count was folded into the running total.
    TotalsAdded { total: u32 },
    /// A file is about to be processed (caller trims to a basename).
    FileStarted { path: String },
    /// A file finished (processed/total advanced).
    FileDone { processed: u32, total: u32 },
    /// Entering the missing-file cleanup phase.
    Cleanup,
    /// Terminal: Complete / Cancelled / Error, with any per-file errors.
    Finished {
        status: ScanStatus,
        errors: Vec<ScanError>,
    },
}
