//! Scan the legacy `tracks/` folder for numeric-named `.flac` files.

use std::path::Path;

/// Detect legacy cached files (numeric FLAC files in tracks/ folder)
pub fn detect_legacy_cached_files(tracks_dir: &Path) -> Result<Vec<u64>, String> {
    log::info!(
        "Scanning for legacy cached files in: {}",
        tracks_dir.display()
    );

    if !tracks_dir.exists() {
        return Ok(Vec::new());
    }

    let mut track_ids = Vec::new();

    match std::fs::read_dir(tracks_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();

                // Only process FLAC files
                if path.extension().and_then(|s| s.to_str()) != Some("flac") {
                    continue;
                }

                // Check if filename is purely numeric (track_id)
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(track_id) = filename.parse::<u64>() {
                        track_ids.push(track_id);
                        log::debug!("Found legacy track: {}", track_id);
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to read tracks directory: {}", e));
        }
    }

    log::info!("Found {} legacy cached files", track_ids.len());
    Ok(track_ids)
}

#[cfg(test)]
mod tests {
    use super::detect_legacy_cached_files;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn missing_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let missing = tmp.path().join("does-not-exist");
        let ids = detect_legacy_cached_files(&missing).unwrap();
        assert!(ids.is_empty());
    }

    #[test]
    fn finds_only_numeric_flac_files() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();

        fs::write(dir.join("12345.flac"), b"fake").unwrap();
        fs::write(dir.join("67890.flac"), b"fake").unwrap();
        // Non-numeric filename — should be ignored.
        fs::write(dir.join("not-a-track-id.flac"), b"fake").unwrap();
        // Non-flac extension — should be ignored even though numeric.
        fs::write(dir.join("11111.mp3"), b"fake").unwrap();

        let mut ids = detect_legacy_cached_files(dir).unwrap();
        ids.sort();
        assert_eq!(ids, vec![12345, 67890]);
    }

    #[test]
    fn empty_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let ids = detect_legacy_cached_files(tmp.path()).unwrap();
        assert!(ids.is_empty());
    }
}
