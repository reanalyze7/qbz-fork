//! Track-number inference from a filename, for files with no usable tag.

use std::path::Path;

use super::MetadataExtractor;

impl MetadataExtractor {
    /// Try to extract a track number from the filename.
    /// Handles common patterns like:
    /// - "01 - Title.flac"
    /// - "01. Title.flac"
    /// - "01_Title.flac"
    /// - "Track 01.flac"
    /// - "1-01 Title.flac" (disc-track)
    pub fn infer_track_number_from_filename(file_path: &Path) -> Option<u32> {
        let stem = file_path.file_stem()?.to_str()?;
        let trimmed = stem.trim();

        // Pattern: starts with digits
        if let Some(cap) = trimmed.strip_prefix(|c: char| c.is_ascii_digit()) {
            // Collect leading digits
            let digit_end = 1 + cap.chars().take_while(|c| c.is_ascii_digit()).count();
            let num_str = &trimmed[..digit_end];
            let rest = &trimmed[digit_end..];

            // Check for disc-track pattern FIRST: "D-TT" like "1-01 Title", "2-05 Song"
            // Only when leading number is 1-2 digits (disc number) followed by dash+digits
            if digit_end <= 2 && rest.starts_with('-') {
                let after_dash = &rest[1..];
                let track_digits: String = after_dash
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if !track_digits.is_empty() {
                    if let Ok(n) = track_digits.parse::<u32>() {
                        if n > 0 && n < 10000 {
                            return Some(n);
                        }
                    }
                }
            }

            // Regular pattern: digits followed by separator
            // "01 - Title", "01. Title", "01_Title", "01-Title"
            let rest_trimmed = rest.trim_start();
            let has_separator = rest_trimmed.starts_with('-')
                || rest_trimmed.starts_with('.')
                || rest_trimmed.starts_with('_')
                || rest_trimmed.starts_with(' ')
                || rest_trimmed.is_empty();

            if has_separator {
                if let Ok(n) = num_str.parse::<u32>() {
                    if n > 0 && n < 10000 {
                        return Some(n);
                    }
                }
            }
        }

        // Pattern: "Track 01" or "Track01"
        let lower = trimmed.to_lowercase();
        if lower.starts_with("track") {
            let after = trimmed[5..].trim_start();
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                if let Ok(n) = digits.parse::<u32>() {
                    if n > 0 && n < 10000 {
                        return Some(n);
                    }
                }
            }
        }

        None
    }
}
