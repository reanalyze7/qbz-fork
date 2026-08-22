//! Pure helpers backing folder-artwork matching: filename normalization and
//! how well a candidate filename matches the album/folder name.

use super::super::MetadataExtractor;

impl MetadataExtractor {
    pub(super) fn normalize_artwork_key(value: &str) -> Option<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return None;
        }
        let normalized: String = trimmed
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized)
        }
    }

    pub(super) fn is_supported_artwork_ext(ext: &str) -> bool {
        matches!(ext, "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp")
    }

    pub(super) fn artwork_score(
        file_key: &str,
        album_key: Option<&str>,
        folder_key: Option<&str>,
    ) -> i32 {
        const EXACT: &[&str] = &["cover", "folder", "front", "album", "artwork", "art"];
        let mut score = 0;

        if EXACT.iter().any(|name| *name == file_key) {
            score = score.max(100);
        }
        if let Some(key) = album_key {
            if file_key == key {
                score = score.max(95);
            } else if file_key.contains(key) {
                score = score.max(70);
            }
        }
        if let Some(key) = folder_key {
            if file_key == key {
                score = score.max(90);
            } else if file_key.contains(key) {
                score = score.max(65);
            }
        }
        if EXACT.iter().any(|name| file_key.contains(name)) {
            score = score.max(80);
        }

        score
    }
}
