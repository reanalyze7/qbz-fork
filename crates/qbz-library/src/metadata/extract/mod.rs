//! Main `LocalTrack`-building entry points.

mod dsd;
mod fallback;
mod properties;
mod tagged;

use lofty::prelude::*;
use lofty::probe::Probe;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::{AudioFormat, LibraryError, LocalTrack};

use super::MetadataExtractor;

/// Shared, already-computed technical properties passed down to the
/// tag-branch and fallback-branch track builders.
pub(super) struct TrackContext {
    pub duration_secs: u64,
    pub sample_rate: f64,
    pub bit_depth: Option<u32>,
    pub channels: u8,
    pub file_size_bytes: u64,
    pub last_modified: i64,
    pub format: AudioFormat,
}

impl MetadataExtractor {
    /// Extract metadata from an audio file
    pub fn extract(file_path: &Path) -> Result<LocalTrack, LibraryError> {
        Self::extract_with_roots(file_path, &[])
    }

    /// Like [`Self::extract`], but the caller supplies the library roots the
    /// file is being scanned under. Roots feed the untagged-artist root
    /// clamp (see `infer_artist_album`); the plain `extract` passes none, so
    /// ephemeral / single-file extraction keeps the legacy inference.
    pub fn extract_with_roots(
        file_path: &Path,
        library_roots: &[PathBuf],
    ) -> Result<LocalTrack, LibraryError> {
        log::debug!("Extracting metadata from: {}", file_path.display());

        // DSD containers aren't lofty-readable: qbz-dsd demuxes them (tech
        // props + embedded ID3v2 for DSF; trailing ID3 for DFF when present).
        if qbz_dsd::is_dsd_path(file_path) {
            return dsd::extract_dsd(file_path, library_roots);
        }

        let tagged_file = Probe::open(file_path)
            .map_err(|e| LibraryError::Metadata(format!("Failed to open file: {}", e)))?
            .read()
            .map_err(|e| LibraryError::Metadata(format!("Failed to read file: {}", e)))?;

        let tag = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag());

        let properties = tagged_file.properties();
        let ctx = TrackContext {
            duration_secs: properties.duration().as_secs(),
            sample_rate: properties.sample_rate().unwrap_or(44100) as f64,
            bit_depth: properties.bit_depth().map(|b| b as u32),
            channels: properties.channels().unwrap_or(2) as u8,
            file_size_bytes: fs::metadata(file_path).map_err(LibraryError::Io)?.len(),
            last_modified: fs::metadata(file_path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            format: Self::detect_format(file_path),
        };

        let filename = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let (fallback_artist, fallback_album) = Self::infer_artist_album(file_path, library_roots);
        let inferred_disc = Self::infer_disc_number(file_path);

        let track = if tag.is_some() {
            tagged::build_track_tagged(
                file_path,
                &tagged_file,
                &ctx,
                filename,
                fallback_artist,
                fallback_album,
                inferred_disc,
            )
        } else {
            fallback::build_track_fallback(
                file_path,
                &ctx,
                filename,
                fallback_artist,
                fallback_album,
                inferred_disc,
            )
        };

        Ok(track)
    }
}
