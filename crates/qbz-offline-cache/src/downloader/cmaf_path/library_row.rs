//! Step 3: fetch metadata + save artwork, then populate the library row.
//!
//! We don't write FLAC tags or embed artwork INSIDE the encrypted blob —
//! that would corrupt it. But we DO save a cover.jpg next to the bundle
//! directory so the library UI has artwork to display.

use qbz_qobuz::cmaf::CmafRawBundle;

use crate::cmaf_store::BundleLayout;

pub(super) async fn populate_library_row(
    track_id: u64,
    client: &std::sync::Arc<tokio::sync::RwLock<Option<qbz_qobuz::QobuzClient>>>,
    library_db: &std::sync::Arc<tokio::sync::Mutex<Option<qbz_library::LibraryDatabase>>>,
    layout: &BundleLayout,
    bundle: &CmafRawBundle,
) {
    let metadata = {
        let client_guard = client.read().await;
        match client_guard.as_ref() {
            Some(qc) => crate::metadata::fetch_complete_metadata(track_id, qc).await,
            None => Err("QobuzClient not initialized".to_string()),
        }
    };
    let metadata = match metadata {
        Ok(metadata) => metadata,
        Err(e) => {
            log::warn!(
                "[Offline/CMAF] Track {} post-metadata fetch failed: {} (bundle already persisted)",
                track_id,
                e
            );
            return;
        }
    };

    // Download and save album artwork alongside the bundle, same as
    // the legacy path does next to the FLAC file. cover.jpg lives at
    // <offline_root>/tracks-cmaf/<track_id>/cover.jpg — set as the
    // library row's artwork_path so the UI picks it up.
    let artwork_path: Option<String> = if let Some(artwork_url) = metadata.artwork_url.as_deref() {
        match crate::metadata::save_album_artwork(&layout.track_dir, artwork_url).await {
            Ok(()) => {
                let cover = layout.track_dir.join("cover.jpg");
                if cover.exists() {
                    Some(cover.to_string_lossy().to_string())
                } else {
                    None
                }
            }
            Err(e) => {
                log::warn!("[Offline/CMAF] Track {} artwork save failed: {}", track_id, e);
                None
            }
        }
    } else {
        None
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
            // For v2 bundles the "playable path" in the library index
            // is the track directory; the player resolves it through
            // the DB's cache_format=2 branch anyway.
            layout.track_dir.to_string_lossy().as_ref(),
            &album_group_key,
            &metadata.album,
            bundle.bit_depth,
            bundle.sampling_rate.map(|r| r as f64),
            artwork_path.as_deref(),
        );
    }
}
