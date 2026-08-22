//! Folder-artwork lookup by filename heuristics (cover.jpg, folder.png, ...).

use std::fs;
use std::path::{Path, PathBuf};

use super::super::MetadataExtractor;

impl MetadataExtractor {
    /// Look for folder artwork by file name heuristics
    pub fn find_folder_artwork(
        audio_file_path: &Path,
        album_title: Option<&str>,
    ) -> Option<String> {
        let parent_dir = audio_file_path.parent()?;
        let album_dir =
            Self::album_root_dir(audio_file_path).unwrap_or_else(|| parent_dir.to_path_buf());

        let mut dirs_to_check: Vec<PathBuf> = Vec::new();
        if album_dir != parent_dir {
            dirs_to_check.push(album_dir.clone());
        }
        dirs_to_check.push(parent_dir.to_path_buf());

        let album_key = album_title
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .filter(|value| !value.eq_ignore_ascii_case("Unknown Album"))
            .map(Self::strip_disc_suffix)
            .and_then(|value| Self::normalize_artwork_key(&value));
        let folder_key = album_dir
            .file_name()
            .and_then(|s| s.to_str())
            .map(Self::strip_disc_suffix)
            .and_then(|value| Self::normalize_artwork_key(&value));

        let mut best: Option<(PathBuf, i32)> = None;
        let mut best_score = 0;
        let mut candidate_count = 0;

        for (index, dir) in dirs_to_check.iter().enumerate() {
            let dir_bonus = if index == 0 { 5 } else { 0 };
            let entries = match fs::read_dir(dir) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if !Self::is_supported_artwork_ext(&ext) {
                    continue;
                }

                candidate_count += 1;
                let file_stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .trim();
                let file_key = match Self::normalize_artwork_key(file_stem) {
                    Some(key) => key,
                    None => {
                        let fallback = file_stem.to_lowercase();
                        if fallback.trim().is_empty() {
                            continue;
                        }
                        fallback
                    }
                };

                let mut score =
                    Self::artwork_score(&file_key, album_key.as_deref(), folder_key.as_deref());
                if score == 0 {
                    score = 5;
                }
                score += dir_bonus;

                if score > best_score {
                    best_score = score;
                    best = Some((path, score));
                }
            }
        }

        if let Some((path, score)) = best {
            if score >= 10 || candidate_count == 1 {
                return Some(path.to_string_lossy().to_string());
            }
        }

        None
    }
}
