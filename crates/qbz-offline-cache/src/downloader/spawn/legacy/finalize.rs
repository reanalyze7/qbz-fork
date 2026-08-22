//! Save cover.jpg, detect bit-depth/sample-rate via `lofty`, insert the
//! library row, update the DB file path, and emit `Processed`.

use crate::event::{CacheEvent, CacheEventSink, CacheFormat};
use crate::metadata::CompleteTrackMetadata;

pub(super) async fn run(
    track_id: u64,
    new_path: &str,
    metadata: &CompleteTrackMetadata,
    library_db: &std::sync::Arc<tokio::sync::Mutex<Option<qbz_library::LibraryDatabase>>>,
    db: &std::sync::Arc<tokio::sync::Mutex<Option<crate::OfflineCacheDb>>>,
    sink: &CacheEventSink,
) {
    // Save cover.jpg next to the organized FLAC so the library
    // UI has artwork to display.
    let artwork_path_v1: Option<String> = if let Some(artwork_url) = metadata.artwork_url.as_deref() {
        if let Some(parent_dir) = std::path::Path::new(new_path).parent() {
            match crate::metadata::save_album_artwork(parent_dir, artwork_url).await {
                Ok(()) => {
                    let cover = parent_dir.join("cover.jpg");
                    if cover.exists() {
                        Some(cover.to_string_lossy().to_string())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    let (bit_depth_detected, sample_rate_detected) = match lofty::read_from_path(new_path) {
        Ok(tagged_file) => {
            use lofty::prelude::*;
            let properties = tagged_file.properties();
            (
                properties.bit_depth().map(|bd| bd as u32),
                properties.sample_rate().map(|sr| sr as f64),
            )
        }
        Err(_) => (None, None),
    };

    let album_artist = metadata.album_artist.as_ref().unwrap_or(&metadata.artist);
    let album_group_key = format!("{}|{}", metadata.album, album_artist);
    let lib_opt = library_db.lock().await;
    if let Some(lib_guard) = lib_opt.as_ref() {
        let _ = lib_guard.insert_qobuz_cached_track_with_grouping(
            track_id,
            &metadata.title,
            &metadata.artist,
            Some(&metadata.album),
            metadata.album_artist.as_deref(),
            metadata.track_number,
            metadata.disc_number,
            metadata.year,
            metadata.duration_secs,
            new_path,
            &album_group_key,
            &metadata.album,
            bit_depth_detected,
            sample_rate_detected,
            artwork_path_v1.as_deref(),
        );
    }

    if let Some(db_guard) = db.lock().await.as_ref() {
        let _ = db_guard.update_file_path(track_id, new_path);
    }

    sink(CacheEvent::Processed {
        track_id,
        path: new_path.to_string(),
        format: CacheFormat::Flac,
    });
}
