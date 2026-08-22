//! Legacy cached file migration service
//!
//! Handles migration of old numeric-named FLAC files to new organized structure.
//!
//! Split into `detect` (directory scan), `single_track` (the per-track
//! migration steps), and `batch` (the public driver iterating track ids).

mod batch;
mod detect;
mod single_track;

pub use batch::migrate_legacy_cached_files;
pub use detect::detect_legacy_cached_files;

use serde::Serialize;

#[derive(Default, Serialize, Clone, Debug)]
pub struct MigrationStatus {
    pub has_legacy_files: bool,
    pub total_tracks: usize,
    pub processed: usize,
    pub successful: usize,
    pub failed: usize,
    pub in_progress: bool,
    pub completed: bool,
    pub errors: Vec<MigrationError>,
}

#[derive(Serialize, Clone, Debug)]
pub struct MigrationError {
    pub track_id: u64,
    pub error_message: String,
}
