//! Lightweight audio-properties extraction and file-extension format
//! detection — no tag reading beyond what lofty/qbz-dsd need for technical
//! properties.

use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::Path;

use crate::{AudioFormat, AudioProperties, LibraryError};

use super::super::MetadataExtractor;

impl MetadataExtractor {
    /// Extract audio properties without full metadata
    pub fn extract_properties(file_path: &Path) -> Result<AudioProperties, LibraryError> {
        if qbz_dsd::is_dsd_path(file_path) {
            let demux = qbz_dsd::open_dsd(file_path)
                .map_err(|e| LibraryError::Metadata(format!("Failed to read DSD file: {}", e)))?;
            let info = demux.info();
            return Ok(AudioProperties {
                duration_secs: info.duration_secs(),
                bit_depth: Some(1),
                sample_rate: info.dsd_rate as f64,
                channels: info.channels as u8,
            });
        }

        let tagged_file = Probe::open(file_path)
            .map_err(|e| LibraryError::Metadata(format!("Failed to open file: {}", e)))?
            .read()
            .map_err(|e| LibraryError::Metadata(format!("Failed to read file: {}", e)))?;

        let properties = tagged_file.properties();

        Ok(AudioProperties {
            duration_secs: properties.duration().as_secs(),
            bit_depth: properties.bit_depth().map(|b| b as u32),
            sample_rate: properties.sample_rate().unwrap_or(44100) as f64,
            channels: properties.channels().unwrap_or(2) as u8,
        })
    }

    /// Determine AudioFormat from file extension
    pub fn detect_format(path: &Path) -> AudioFormat {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("flac") => AudioFormat::Flac,
            Some("m4a") => AudioFormat::Alac,
            Some("wav") => AudioFormat::Wav,
            Some("aiff") | Some("aif") => AudioFormat::Aiff,
            Some("ape") => AudioFormat::Ape,
            Some("mp3") => AudioFormat::Mp3,
            Some("dsf") | Some("dff") => AudioFormat::Dsd,
            _ => AudioFormat::Unknown,
        }
    }
}
