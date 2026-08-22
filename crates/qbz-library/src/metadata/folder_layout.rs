//! Folder-structure inference: recognizing encoding/quality subfolders and
//! disc subfolders, and walking up past them to find the album root dir.

use std::path::{Path, PathBuf};

use super::MetadataExtractor;

impl MetadataExtractor {
    /// Returns true if the folder name looks like an audio encoding/quality
    /// directory (e.g., "FLAC 24-bit - 96 kHz", "MP3 320 kbps").
    pub(super) fn is_encoding_folder(name: &str) -> bool {
        let lower = name.to_lowercase();
        let first_word = lower
            .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
            .find(|tok| !tok.is_empty());

        if let Some(word) = first_word {
            if matches!(
                word,
                "flac"
                    | "mp3"
                    | "aac"
                    | "alac"
                    | "wav"
                    | "aiff"
                    | "ogg"
                    | "dsd"
                    | "opus"
                    | "wma"
                    | "ape"
                    | "pcm"
            ) {
                return true;
            }
        }

        // Standalone bitrate patterns like "320kbps"
        if lower.contains("kbps") {
            return true;
        }

        false
    }

    pub(super) fn album_root_dir(file_path: &Path) -> Option<PathBuf> {
        let mut dir = file_path.parent()?.to_path_buf();

        // Skip past disc and encoding subdirectories to find the actual album root.
        // Handles: album/track, album/disc1/track, album/FLAC 24-96/track,
        //          album/FLAC 24-96/disc1/track
        for _ in 0..2 {
            let name = dir.file_name().and_then(|s| s.to_str());
            match name {
                Some(n) if Self::is_disc_folder(n) || Self::is_encoding_folder(n) => {
                    dir = dir.parent()?.to_path_buf();
                }
                _ => break,
            }
        }

        Some(dir)
    }

    pub fn infer_disc_number(file_path: &Path) -> Option<u32> {
        let parent_dir = file_path.parent()?;
        let parent_name = parent_dir.file_name()?.to_str()?;
        if !Self::is_disc_folder(parent_name) {
            return None;
        }
        Self::disc_number_from_name(parent_name)
    }
}
