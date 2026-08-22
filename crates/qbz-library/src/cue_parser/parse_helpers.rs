use crate::LibraryError;

use super::model::CueSheet;
use super::parse::CueParser;

impl CueParser {
    /// Validate the parsed sheet has an audio file and at least one track,
    /// then log a summary.
    pub(super) fn validate_and_log(sheet: CueSheet) -> Result<CueSheet, LibraryError> {
        if sheet.audio_file.is_empty() {
            return Err(LibraryError::CueParse(
                "No FILE directive found in CUE sheet".to_string(),
            ));
        }

        if sheet.tracks.is_empty() {
            return Err(LibraryError::CueParse(
                "No tracks found in CUE sheet".to_string(),
            ));
        }

        log::info!(
            "Parsed CUE: {} tracks, audio file: {}",
            sheet.tracks.len(),
            sheet.audio_file
        );

        Ok(sheet)
    }

    /// Extract quoted string: COMMAND "value" -> value
    pub(super) fn extract_quoted(line: &str) -> Option<String> {
        let start = line.find('"')?;
        let end = line.rfind('"')?;
        if end <= start {
            return None;
        }
        Some(line[start + 1..end].to_string())
    }

    /// Extract track number: "TRACK 01 AUDIO" -> 1
    pub(super) fn extract_track_number(line: &str) -> Option<u32> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }
        parts[1].parse().ok()
    }
}
