//! Build the final `<artist>/<album>/[Disc N/]NN - Title.flac` path and
//! move the downloaded temp file into place.

use std::path::Path;

use super::filename::sanitize_filename;
use super::model::CompleteTrackMetadata;

/// Organize cached file into proper folder structure
pub fn organize_cached_file(
    track_id: u64,
    temp_path: &str,
    root_dir: &str,
    metadata: &CompleteTrackMetadata,
) -> Result<String, String> {
    log::info!("Organizing cached file for track {}", track_id);

    let temp = Path::new(temp_path);
    let root = Path::new(root_dir);

    // Build target path: <root>/<artist>/<album>/[Disc N/]NN - Title.flac
    let artist_dir = sanitize_filename(metadata.album_artist.as_ref().unwrap_or(&metadata.artist));
    let album_dir = sanitize_filename(&metadata.album);

    let mut target_dir = root.join(&artist_dir).join(&album_dir);

    // Add disc subfolder if multi-disc
    if let Some(disc) = metadata.disc_number {
        if disc > 1 {
            target_dir = target_dir.join(format!("Disc {}", disc));
        }
    }

    // Create directory structure
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create directories: {}", e))?;

    // Build filename: NN - Title.flac
    let track_num = metadata.track_number.unwrap_or(0);
    let title_clean = sanitize_filename(&metadata.title);
    let filename = if track_num > 0 {
        format!("{:02} - {}.flac", track_num, title_clean)
    } else {
        format!("{}.flac", title_clean)
    };

    let target_path = target_dir.join(&filename);

    // Handle filename conflicts
    let final_path = if target_path.exists() {
        let mut counter = 2;
        loop {
            let alt_filename = if track_num > 0 {
                format!("{:02} - {} ({}).flac", track_num, title_clean, counter)
            } else {
                format!("{} ({}).flac", title_clean, counter)
            };
            let alt_path = target_dir.join(&alt_filename);
            if !alt_path.exists() {
                break alt_path;
            }
            counter += 1;
            if counter > 100 {
                return Err("Too many filename conflicts".to_string());
            }
        }
    } else {
        target_path
    };

    // Move file
    std::fs::rename(temp, &final_path).map_err(|e| format!("Failed to move file: {}", e))?;

    Ok(final_path.to_string_lossy().to_string())
}
