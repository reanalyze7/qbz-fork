use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{AlbumTagSidecar, LibraryDatabase, ScanError, ScanStatus};

use super::audio_file::process_audio_file;
use super::event::ScanEvent;
use super::helpers::normalize_path;
use super::outcome::ScanOutcome;

/// Process a folder's audio files, skipping any already covered by a CUE
/// sheet, checking `cancel` at every file boundary. `sidecar_cache` spans the
/// whole scan; `folder_artwork_cache` is scoped to this one folder.
#[allow(clippy::too_many_arguments)]
pub(super) fn process_audio_files(
    db: &LibraryDatabase,
    audio_files: &[PathBuf],
    cue_audio_files: &HashSet<String>,
    folder_root: &PathBuf,
    artwork_cache: &Path,
    cancel: &AtomicBool,
    on_event: &(dyn Fn(ScanEvent) + Send + Sync),
    all_errors: &mut Vec<ScanError>,
    sidecar_cache: &mut HashMap<String, Option<AlbumTagSidecar>>,
    total: u32,
    processed: &mut u32,
) -> ScanOutcome {
    let mut folder_artwork_cache = HashMap::new();

    for audio_path in audio_files {
        if cancel.load(Ordering::Relaxed) {
            on_event(ScanEvent::Finished {
                status: ScanStatus::Cancelled,
                errors: std::mem::take(all_errors),
            });
            return ScanOutcome::Cancelled;
        }

        let canonical = normalize_path(audio_path);
        let path_str = canonical.to_string_lossy().to_string();
        if cue_audio_files.contains(&path_str) {
            *processed += 1;
            on_event(ScanEvent::FileDone {
                processed: *processed,
                total,
            });
            continue;
        }

        on_event(ScanEvent::FileStarted {
            path: path_str.clone(),
        });

        if let Err(e) = process_audio_file(
            db,
            &canonical,
            folder_root,
            artwork_cache,
            sidecar_cache,
            &mut folder_artwork_cache,
        ) {
            all_errors.push(ScanError {
                file_path: path_str,
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
