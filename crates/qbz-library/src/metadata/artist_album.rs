//! Artist/album inference from folder structure, and album-grouping key
//! derivation.

use std::path::{Path, PathBuf};

use super::MetadataExtractor;

impl MetadataExtractor {
    pub(super) fn infer_artist_album(
        file_path: &Path,
        library_roots: &[PathBuf],
    ) -> (Option<String>, Option<String>) {
        let album_dir = Self::album_root_dir(file_path);
        let album_name = album_dir
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .map(Self::strip_year_suffix);

        let artist_name = album_dir
            .as_ref()
            .and_then(|p| p.parent())
            .and_then(|parent| {
                // Root clamp (spec 2026-07-19-local-album-grouping-mode §C):
                // an album dir hanging DIRECTLY off a library root means the
                // "parent folder" IS the root itself — its name ("Music", …)
                // is structural, never an artist. Untagged albums at root
                // level (e.g. a tagless DSD set) used to surface with the
                // root's name as the artist.
                if library_roots.iter().any(|root| root == parent) {
                    None
                } else {
                    parent
                        .file_name()
                        .and_then(|s| s.to_str())
                        .map(Self::strip_year_suffix)
                }
            });

        if artist_name.is_none() {
            if let Some(album_dir_name) = album_name.as_deref() {
                if let Some((artist, album)) = album_dir_name.split_once(" - ") {
                    return (
                        Some(Self::strip_year_suffix(artist)),
                        Some(Self::strip_year_suffix(album)),
                    );
                }
            }
        }

        (artist_name, album_name)
    }

    pub fn album_group_info(file_path: &Path, tag_album: Option<&str>) -> (String, String) {
        let album_dir = Self::album_root_dir(file_path);
        let group_key = album_dir
            .as_ref()
            .map(|dir| dir.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.to_string_lossy().to_string());

        let mut group_title = tag_album
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .filter(|value| !value.eq_ignore_ascii_case("Unknown Album"))
            .map(|value| value.to_string())
            .or_else(|| {
                album_dir
                    .as_ref()
                    .and_then(|dir| dir.file_name())
                    .and_then(|s| s.to_str())
                    .map(Self::strip_year_suffix)
            })
            .unwrap_or_else(|| "Unknown Album".to_string());

        group_title = Self::strip_disc_suffix(&group_title);

        (group_key, group_title)
    }
}
