//! Migrate one legacy cached track: fetch metadata → write tags → embed
//! artwork → organize file → save album artwork → read audio properties →
//! insert into the library DB. Steps are numbered per the original
//! sequential comments; each depends on the previous step's output
//! (`new_path`, `metadata`) so this stays one function.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::metadata::{
    embed_artwork, fetch_complete_metadata, organize_cached_file, save_album_artwork,
    write_flac_tags,
};
use qbz_library::LibraryDatabase;
use qbz_qobuz::QobuzClient;

/// Migrate a single legacy cached track
pub(super) async fn migrate_single_track(
    track_id: u64,
    legacy_path: PathBuf,
    offline_root: &str,
    qobuz_client: &QobuzClient,
    library_db: Arc<Mutex<Option<LibraryDatabase>>>,
) -> Result<String, String> {
    log::info!("Migrating track {}", track_id);

    // 1. Fetch complete metadata from Qobuz
    let metadata = fetch_complete_metadata(track_id, qobuz_client).await?;

    // 2. Write FLAC tags
    let legacy_path_str = legacy_path.to_string_lossy().to_string();
    write_flac_tags(&legacy_path_str, &metadata)
        .map_err(|e| format!("Failed to write tags: {}", e))?;

    // 3. Embed artwork if available
    if let Some(artwork_url) = &metadata.artwork_url {
        if let Err(e) = embed_artwork(&legacy_path_str, artwork_url).await {
            log::warn!("Failed to embed artwork for track {}: {}", track_id, e);
        }
    }

    // 4. Organize file into artist/album structure
    let new_path = organize_cached_file(track_id, &legacy_path_str, offline_root, &metadata)?;

    // 5. Save album artwork as cover.jpg
    let artwork_path = if let Some(artwork_url) = &metadata.artwork_url {
        if let Some(parent_dir) = std::path::Path::new(&new_path).parent() {
            match save_album_artwork(parent_dir, artwork_url).await {
                Ok(_) => Some(parent_dir.join("cover.jpg").to_string_lossy().to_string()),
                Err(e) => {
                    log::warn!("Failed to save album artwork for track {}: {}", track_id, e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // 6. Extract audio properties from FLAC file
    use lofty::prelude::*;
    let (bit_depth, sample_rate) = match lofty::read_from_path(&new_path) {
        Ok(tagged_file) => {
            let properties = tagged_file.properties();
            let bit_depth = properties.bit_depth().map(|bd| bd as u32);
            let sample_rate = properties.sample_rate().map(|sr| sr as f64);
            (bit_depth, sample_rate)
        }
        Err(e) => {
            log::warn!(
                "Failed to read audio properties for track {}: {}",
                track_id,
                e
            );
            (None, None)
        }
    };

    // 7. Insert into local library DB
    let lib_opt__ = library_db.lock().await;
    let lib_guard = lib_opt__
        .as_ref()
        .ok_or("No active session - please log in")?;

    let album_artist = metadata.album_artist.as_ref().unwrap_or(&metadata.artist);
    let album_group_key = format!("{}|{}", metadata.album, album_artist);

    lib_guard
        .insert_qobuz_cached_track_with_grouping(
            track_id,
            &metadata.title,
            &metadata.artist,
            Some(&metadata.album),
            metadata.album_artist.as_deref(),
            metadata.track_number,
            metadata.disc_number,
            metadata.year,
            metadata.duration_secs,
            &new_path,
            &album_group_key,
            &metadata.album,
            bit_depth,
            sample_rate,
            artwork_path.as_deref(),
        )
        .map_err(|e| format!("Failed to insert to library DB: {}", e))?;

    log::info!("Track {} migrated successfully to: {}", track_id, new_path);
    Ok(new_path)
}
