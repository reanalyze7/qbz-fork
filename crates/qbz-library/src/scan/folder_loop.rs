use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use crate::{AlbumTagSidecar, CueParser, LibraryDatabase, LibraryFolder, LibraryScanner, ScanError};

use super::audio_loop::process_audio_files;
use super::cue_loop::process_cue_files;
use super::event::ScanEvent;
use super::helpers::normalize_path;
use super::outcome::ScanOutcome;

/// Scan one folder: CUE files first (one file -> several virtual tracks),
/// then audio files (skipping any referenced by a CUE sheet). `sidecar_cache`
/// spans the whole scan.
#[allow(clippy::too_many_arguments)]
pub(super) fn scan_folder(
    db: &LibraryDatabase,
    folder: &LibraryFolder,
    scanner: &LibraryScanner,
    artwork_cache: &Path,
    cancel: &AtomicBool,
    on_event: &(dyn Fn(ScanEvent) + Send + Sync),
    all_errors: &mut Vec<ScanError>,
    sidecar_cache: &mut HashMap<String, Option<AlbumTagSidecar>>,
    total: &mut u32,
    processed: &mut u32,
) -> ScanOutcome {
    // The folder's own normalized path, for the untagged-artist root clamp
    // (an album dir directly under THIS root must not inherit the root's
    // name as the artist — spec §C).
    let folder_root = normalize_path(Path::new(&folder.path));
    let scan_result = match scanner.scan_directory(Path::new(&folder.path)) {
        Ok(r) => r,
        Err(e) => {
            all_errors.push(ScanError {
                file_path: folder.path.clone(),
                error: e.to_string(),
            });
            return ScanOutcome::Continue;
        }
    };

    *total += (scan_result.audio_files.len() + scan_result.cue_files.len()) as u32;
    on_event(ScanEvent::TotalsAdded { total: *total });

    if let ScanOutcome::Cancelled = process_cue_files(
        db,
        &scan_result.cue_files,
        artwork_cache,
        cancel,
        on_event,
        all_errors,
        *total,
        processed,
    ) {
        return ScanOutcome::Cancelled;
    }

    let cue_audio_files: HashSet<String> = scan_result
        .cue_files
        .iter()
        .filter_map(|p| {
            CueParser::parse(p).ok().map(|cue| {
                normalize_path(Path::new(&cue.audio_file))
                    .to_string_lossy()
                    .to_string()
            })
        })
        .collect();

    process_audio_files(
        db,
        &scan_result.audio_files,
        &cue_audio_files,
        &folder_root,
        artwork_cache,
        cancel,
        on_event,
        all_errors,
        sidecar_cache,
        *total,
        processed,
    )
}
