use std::fs;
use std::path::Path;

use crate::LibraryError;

use super::model::{CueSheet, CueTime, CueTrack};

/// CUE sheet parser
pub struct CueParser;

impl CueParser {
    /// Parse a CUE file
    pub fn parse(cue_path: &Path) -> Result<CueSheet, LibraryError> {
        log::debug!("Parsing CUE file: {}", cue_path.display());

        // Try UTF-8 first, then fall back to Latin-1
        let content = fs::read_to_string(cue_path).or_else(|_| {
            let bytes = fs::read(cue_path)?;
            Ok::<String, std::io::Error>(bytes.iter().map(|&b| b as char).collect())
        })?;

        Self::parse_content(&content, cue_path)
    }

    /// Parse CUE content
    fn parse_content(content: &str, cue_path: &Path) -> Result<CueSheet, LibraryError> {
        let mut sheet = CueSheet {
            file_path: cue_path.to_string_lossy().to_string(),
            audio_file: String::new(),
            title: None,
            performer: None,
            tracks: Vec::new(),
        };

        let mut current_track: Option<CueTrack> = None;
        let mut in_track = false;

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("REM") {
                continue;
            }

            // Parse FILE "name" TYPE
            if line.to_uppercase().starts_with("FILE ") {
                if let Some(filename) = Self::extract_quoted(line) {
                    // Resolve path relative to CUE file
                    if let Some(parent) = cue_path.parent() {
                        let audio_path = parent.join(&filename);
                        sheet.audio_file = audio_path.to_string_lossy().to_string();
                    } else {
                        sheet.audio_file = filename;
                    }
                }
            }
            // Parse album-level TITLE (before any TRACK)
            else if line.to_uppercase().starts_with("TITLE ") && !in_track {
                sheet.title = Self::extract_quoted(line);
            }
            // Parse album-level PERFORMER (before any TRACK)
            else if line.to_uppercase().starts_with("PERFORMER ") && !in_track {
                sheet.performer = Self::extract_quoted(line);
            }
            // Parse TRACK NN AUDIO
            else if line.to_uppercase().starts_with("TRACK ") {
                // Save previous track
                if let Some(track) = current_track.take() {
                    sheet.tracks.push(track);
                }

                // Start new track
                in_track = true;
                if let Some(num) = Self::extract_track_number(line) {
                    current_track = Some(CueTrack {
                        number: num,
                        title: format!("Track {}", num),
                        performer: None,
                        start_secs: 0.0,
                    });
                }
            }
            // Parse track TITLE
            else if line.to_uppercase().starts_with("TITLE ") && in_track {
                if let Some(ref mut track) = current_track {
                    if let Some(title) = Self::extract_quoted(line) {
                        track.title = title;
                    }
                }
            }
            // Parse track PERFORMER
            else if line.to_uppercase().starts_with("PERFORMER ") && in_track {
                if let Some(ref mut track) = current_track {
                    track.performer = Self::extract_quoted(line);
                }
            }
            // Parse INDEX 01 MM:SS:FF (track start time)
            else if line.to_uppercase().starts_with("INDEX 01 ") {
                if let Some(ref mut track) = current_track {
                    let time_str = line.get(9..).map(|s| s.trim()).unwrap_or("");
                    if let Some(time) = CueTime::parse(time_str) {
                        track.start_secs = time.to_seconds();
                    }
                }
            }
        }

        // Don't forget the last track
        if let Some(track) = current_track {
            sheet.tracks.push(track);
        }

        Self::validate_and_log(sheet)
    }
}
