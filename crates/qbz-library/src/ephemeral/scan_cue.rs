//! CUE-sheet scan pass: explode each CUE file into its virtual tracks.
//!
//! Runs before `scan_audio` so its claimed-files set
//! (`cue_referenced_audio`) exists before the plain-audio pass, which
//! uses it to avoid double-listing audio files already exploded here.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use super::EphemeralLibraryInner;
use crate::{cue_to_tracks, CueParser, LocalTrack, MetadataExtractor, ScanResult};

/// Scan every CUE file in `scan`, assign synthetic ids via `inner`, and
/// return the resulting tracks, a skipped-file count, and the set of
/// canonical audio paths claimed by a CUE sheet.
pub(super) fn scan_cue_files(
    scan: &ScanResult,
    inner: &mut EphemeralLibraryInner,
    artwork_cache: &Path,
    album_artwork_cache: &mut HashMap<String, Option<String>>,
) -> (Vec<LocalTrack>, usize, HashSet<PathBuf>) {
    let mut tracks_out: Vec<LocalTrack> = Vec::new();
    let mut skipped_files: usize = 0;
    let mut cue_referenced_audio: HashSet<PathBuf> = HashSet::new();

    for cue_path in &scan.cue_files {
        match CueParser::parse(cue_path) {
            Ok(mut cue) => {
                let audio_path_raw = Path::new(&cue.audio_file).to_path_buf();
                let canonical = std::fs::canonicalize(&audio_path_raw)
                    .unwrap_or_else(|_| audio_path_raw.clone());
                if !canonical.exists() {
                    log::warn!(
                        "[ephemeral] CUE references missing audio: {} -> {}",
                        cue_path.display(),
                        audio_path_raw.display()
                    );
                    skipped_files += 1;
                    continue;
                }
                cue.audio_file = canonical.to_string_lossy().to_string();

                // Symphonia (the play_data decoder) covers FLAC / MP3 /
                // M4A (AAC + ALAC) / WAV / AIFF out of the box; it can't
                // handle APE or raw BIN. Skip CUE files pointing at those
                // — better an empty pane than a track that explodes on click.
                let ext_lower = canonical
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase());
                let playable_via_cue = matches!(
                    ext_lower.as_deref(),
                    Some("flac" | "mp3" | "m4a" | "alac" | "wav" | "aiff" | "aif")
                );
                if !playable_via_cue {
                    log::warn!(
                        "[ephemeral] CUE references unsupported audio format ({:?}) — skipping: {}",
                        ext_lower,
                        canonical.display()
                    );
                    skipped_files += 1;
                    continue;
                }

                let properties = match MetadataExtractor::extract_properties(&canonical) {
                    Ok(p) => p,
                    Err(e) => {
                        log::warn!(
                            "[ephemeral] failed to read audio properties for {}: {}",
                            canonical.display(),
                            e
                        );
                        skipped_files += 1;
                        continue;
                    }
                };
                let format = MetadataExtractor::detect_format(&canonical);

                let mut cue_tracks =
                    cue_to_tracks(&cue, properties.duration_secs, format, &properties);
                if cue_tracks.is_empty() {
                    log::warn!("[ephemeral] CUE produced no tracks: {}", cue_path.display());
                    skipped_files += 1;
                    continue;
                }

                // CUE = single album: resolve cover once, share across
                // every CUE-derived track. Key falls back to the CUE
                // path when album_group_key is empty (no TITLE/PERFORMER).
                let album_key = if !cue_tracks[0].album_group_key.is_empty() {
                    cue_tracks[0].album_group_key.clone()
                } else {
                    format!("cue:{}", cue.file_path)
                };
                let artwork = if let Some(cached) = album_artwork_cache.get(&album_key) {
                    cached.clone()
                } else {
                    let mut found = MetadataExtractor::extract_artwork(&canonical, artwork_cache);
                    if found.is_none() {
                        if let Some(folder_art) =
                            MetadataExtractor::find_folder_artwork(&canonical, cue.title.as_deref())
                        {
                            found = MetadataExtractor::cache_artwork_file(
                                Path::new(&folder_art),
                                artwork_cache,
                            );
                        }
                    }
                    album_artwork_cache.insert(album_key, found.clone());
                    found
                };

                for mut track in cue_tracks.drain(..) {
                    track.id = inner.next_id;
                    inner.next_id += 1;
                    track.source = Some("ephemeral".to_string());
                    track.artwork_path = artwork.clone();
                    inner.tracks.insert(track.id, track.clone());
                    tracks_out.push(track);
                }
                cue_referenced_audio.insert(canonical);
            }
            Err(e) => {
                log::warn!("[ephemeral] failed to parse CUE {}: {}", cue_path.display(), e);
                skipped_files += 1;
            }
        }
    }

    (tracks_out, skipped_files, cue_referenced_audio)
}
