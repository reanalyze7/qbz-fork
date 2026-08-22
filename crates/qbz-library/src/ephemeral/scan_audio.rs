//! Plain-audio-file scan pass: extract metadata for files not already
//! claimed by a CUE sheet in `scan_cue`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::EphemeralLibraryInner;
use crate::{LocalTrack, MetadataExtractor, ScanResult};

/// Scan every plain audio file in `scan` not present in
/// `cue_referenced_audio`, assign synthetic ids via `inner`, and return
/// the resulting tracks plus a skipped-file count.
pub(super) fn scan_audio_files(
    scan: &ScanResult,
    inner: &mut EphemeralLibraryInner,
    artwork_cache: &Path,
    album_artwork_cache: &mut HashMap<String, Option<String>>,
    folder_artwork_cache: &mut HashMap<PathBuf, Option<String>>,
    cue_referenced_audio: &HashSet<PathBuf>,
) -> (Vec<LocalTrack>, usize) {
    let mut tracks_out: Vec<LocalTrack> = Vec::with_capacity(scan.audio_files.len());
    let mut skipped_files: usize = 0;

    for audio_file in &scan.audio_files {
        // Skip audio files that were already exploded into tracks via
        // a CUE sheet — listing them again as a single row would
        // duplicate the album and confuse playback (the CUE-derived
        // track ids are the canonical ones).
        let canonical_audio =
            std::fs::canonicalize(audio_file).unwrap_or_else(|_| audio_file.clone());
        if cue_referenced_audio.contains(&canonical_audio) {
            continue;
        }

        // The scanner accepts APE because the regular library tracks
        // them for tag/metadata purposes, but Symphonia can't decode
        // Monkey's Audio. In ephemeral mode there is no value in
        // surfacing rows that explode on click — skip them so the
        // pane only shows tracks the user can actually play.
        let ext_lower = audio_file
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        if matches!(ext_lower.as_deref(), Some("ape")) {
            log::info!(
                "[ephemeral] skipping APE (no Symphonia decoder): {}",
                audio_file.display()
            );
            skipped_files += 1;
            continue;
        }
        match MetadataExtractor::extract(audio_file) {
            Ok(mut track) => {
                track.id = inner.next_id;
                inner.next_id += 1;
                track.source = Some("ephemeral".to_string());

                let album_key = if !track.album_group_key.is_empty() {
                    track.album_group_key.clone()
                } else {
                    format!(
                        "{}|||{}",
                        track.album,
                        track.album_artist.as_deref().unwrap_or(&track.artist)
                    )
                };

                let artwork = if let Some(cached) = album_artwork_cache.get(&album_key) {
                    cached.clone()
                } else {
                    let mut found = MetadataExtractor::extract_artwork(audio_file, artwork_cache);
                    if found.is_none() {
                        let folder_key = audio_file
                            .parent()
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(|| audio_file.to_path_buf());
                        let folder_art = folder_artwork_cache
                            .entry(folder_key)
                            .or_insert_with(|| {
                                MetadataExtractor::find_folder_artwork(
                                    audio_file,
                                    Some(track.album.as_str()),
                                )
                            })
                            .clone();
                        if let Some(folder_art) = folder_art {
                            found = MetadataExtractor::cache_artwork_file(
                                std::path::Path::new(&folder_art),
                                artwork_cache,
                            );
                        }
                    }
                    album_artwork_cache.insert(album_key, found.clone());
                    found
                };
                track.artwork_path = artwork;

                inner.tracks.insert(track.id, track.clone());
                tracks_out.push(track);
            }
            Err(e) => {
                log::warn!(
                    "[ephemeral] failed to extract metadata from {}: {}",
                    audio_file.display(),
                    e
                );
                skipped_files += 1;
            }
        }
    }

    (tracks_out, skipped_files)
}
