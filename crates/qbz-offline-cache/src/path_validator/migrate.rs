//! Move cached FLAC files from an old offline root to a new one when the
//! user relocates the cache. v1 (plain-FLAC) only — see module doc.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::validate::{validate_path, PathStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveReport {
    pub total_files: usize,
    pub moved_count: usize,
    pub failed_files: Vec<String>,
}

/// Move cached files from old path to new path
pub fn move_cached_files_to_new_path(old_root: &str, new_root: &str) -> Result<MoveReport, String> {
    let old_path = Path::new(old_root);
    let new_path = Path::new(new_root);

    // Validate new path
    let validation = validate_path(new_root)?;
    if !matches!(validation.status, PathStatus::Valid) {
        return Err(format!("New path is not valid: {}", validation.message));
    }

    // Create new root if it doesn't exist
    fs::create_dir_all(new_path).map_err(|e| format!("Failed to create new directory: {}", e))?;

    let mut report = MoveReport {
        total_files: 0,
        moved_count: 0,
        failed_files: Vec::new(),
    };

    // Collect all FLAC files recursively
    let files = collect_flac_files(old_path)?;
    report.total_files = files.len();

    for old_file in files {
        // Get relative path from old root
        let relative_path = old_file
            .strip_prefix(old_path)
            .map_err(|e| format!("Failed to get relative path: {}", e))?;

        let new_file = new_path.join(relative_path);

        // Create parent directories in new location
        if let Some(parent) = new_file.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Move file
        match fs::rename(&old_file, &new_file) {
            Ok(_) => {
                report.moved_count += 1;
            }
            Err(e) => {
                log::warn!("Failed to move file {:?}: {}", old_file, e);
                report
                    .failed_files
                    .push(old_file.to_string_lossy().to_string());
            }
        }
    }

    Ok(report)
}

/// Recursively collect all FLAC files in a directory
fn collect_flac_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    if !dir.is_dir() {
        return Ok(files);
    }

    let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            // Recurse into subdirectories
            files.extend(collect_flac_files(&path)?);
        } else if path.extension().and_then(|s| s.to_str()) == Some("flac") {
            files.push(path);
        }
    }

    Ok(files)
}
