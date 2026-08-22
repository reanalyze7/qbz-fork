//! Fetch metadata, write FLAC tags, embed artwork, and organize the file
//! into its final `<artist>/<album>/NN - Title.flac` location.
//!
//! Returns `None` (logging why) if metadata fetch or file organizing
//! fails — both are unrecoverable for this track's post-processing. Tag
//! write and artwork embed failures are logged and swallowed (best effort).

use crate::metadata::CompleteTrackMetadata;

pub(super) async fn run(
    track_id: u64,
    file_path: &std::path::Path,
    client: &std::sync::Arc<tokio::sync::RwLock<Option<qbz_qobuz::QobuzClient>>>,
    offline_root: &str,
) -> Option<(String, CompleteTrackMetadata)> {
    let file_path_str = file_path.to_string_lossy().to_string();

    let metadata = {
        let client_guard = client.read().await;
        let result = match client_guard.as_ref() {
            Some(qc) => crate::metadata::fetch_complete_metadata(track_id, qc).await,
            None => Err("QobuzClient not initialized".to_string()),
        };
        match result {
            Ok(m) => m,
            Err(e) => {
                log::warn!(
                    "Post-processing metadata fetch failed for {}: {}",
                    track_id,
                    e
                );
                return None;
            }
        }
    };

    if let Err(e) = crate::metadata::write_flac_tags(&file_path_str, &metadata) {
        log::warn!("Failed to write tags for {}: {}", track_id, e);
    }
    if let Some(artwork_url) = &metadata.artwork_url {
        if let Err(e) = crate::metadata::embed_artwork(&file_path_str, artwork_url).await {
            log::warn!("Failed to embed artwork for {}: {}", track_id, e);
        }
    }

    let new_path = match crate::metadata::organize_cached_file(
        track_id,
        &file_path_str,
        offline_root,
        &metadata,
    ) {
        Ok(p) => p,
        Err(e) => {
            log::warn!("Failed to organize cached file {}: {}", track_id, e);
            return None;
        }
    };

    Some((new_path, metadata))
}
