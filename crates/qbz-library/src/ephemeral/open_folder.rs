//! `open_folder`: orchestrates the CUE and plain-audio scan passes and
//! finalizes the resulting track list.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::scan_audio::scan_audio_files;
use super::scan_cue::scan_cue_files;
use super::{EphemeralError, EphemeralFolderResult, EphemeralLibraryState};
use crate::LibraryScanner;

impl EphemeralLibraryState {
    /// Scan a folder, extract metadata for every supported audio file
    /// found, assign synthetic high ids and stash the result. The
    /// previous ephemeral session, if any, is dropped.
    pub fn open_folder(&self, path: &Path) -> Result<EphemeralFolderResult, EphemeralError> {
        if !path.exists() {
            return Err(EphemeralError::Io(format!(
                "Folder does not exist: {}",
                path.display()
            )));
        }
        if !path.is_dir() {
            return Err(EphemeralError::Io(format!(
                "Not a directory: {}",
                path.display()
            )));
        }

        let scanner = LibraryScanner::new();
        let scan = scanner.scan_directory(path)?;

        let mut inner = self.inner.lock().map_err(|_| EphemeralError::Lock)?;
        inner.reset();

        // Cache directory for artwork thumbnails. Same one the regular
        // index uses, so ephemeral artwork piggy-backs on the existing
        // thumbnail pipeline (and gets evicted by the same housekeeping).
        let artwork_cache = crate::get_artwork_cache_dir();

        // Two artwork caches keyed at different granularities. The bigger
        // win is the album-level cache: embedded covers are usually
        // identical across every track of an album, so doing extract_artwork
        // (Probe::open + thumbnail encode) 155 times for a 155-track album
        // is wasted I/O. The folder-level cache is a smaller secondary
        // saver for find_folder_artwork (cover.jpg lookup) when albums
        // share the same parent directory.
        let mut album_artwork_cache: HashMap<String, Option<String>> = HashMap::new();
        let mut folder_artwork_cache: HashMap<PathBuf, Option<String>> = HashMap::new();

        // CUE first: audio files it references get added to
        // cue_referenced_audio so the plain-audio pass below can skip
        // them (otherwise the user would see both the CUE-derived
        // tracks and a single-row entry for the underlying FLAC/WAV).
        let (mut tracks_out, mut skipped_files, cue_referenced_audio) = scan_cue_files(
            &scan,
            &mut inner,
            &artwork_cache,
            &mut album_artwork_cache,
        );

        let (audio_tracks, audio_skipped) = scan_audio_files(
            &scan,
            &mut inner,
            &artwork_cache,
            &mut album_artwork_cache,
            &mut folder_artwork_cache,
            &cue_referenced_audio,
        );
        tracks_out.extend(audio_tracks);
        skipped_files += audio_skipped;

        // Musical order (album, then disc/track/title — same as the DB-backed
        // folder view): the extraction order above is readdir order, which is
        // arbitrary. Ids must FOLLOW the display order because
        // `tracks_snapshot` builds play queues by id — so this call's entries
        // are re-keyed after sorting.
        tracks_out.sort_by(|a, b| {
            a.album_group_key
                .cmp(&b.album_group_key)
                .then_with(|| a.disc_number.unwrap_or(1).cmp(&b.disc_number.unwrap_or(1)))
                .then_with(|| {
                    a.track_number
                        .unwrap_or(u32::MAX)
                        .cmp(&b.track_number.unwrap_or(u32::MAX))
                })
                .then_with(|| a.title.cmp(&b.title))
        });
        for track in &tracks_out {
            inner.tracks.remove(&track.id);
        }
        for track in &mut tracks_out {
            track.id = inner.next_id;
            inner.next_id += 1;
            inner.tracks.insert(track.id, track.clone());
        }

        let folder_path = path.display().to_string();
        inner.current_folder_path = Some(folder_path.clone());

        log::info!(
            "[ephemeral] opened {} ({} tracks, {} skipped)",
            folder_path,
            tracks_out.len(),
            skipped_files
        );

        Ok(EphemeralFolderResult {
            folder_path,
            tracks: tracks_out,
            skipped_files,
        })
    }
}
