//! `spawn_track_cache_download`: the shared tokio::spawn wrapper used by
//! both single-track and batch offline-cache queuing.
//!
//! Acquires the concurrency-limiting semaphore permit (`permit`), tries the
//! CMAF path first (`super::cmaf_path::try_cmaf_offline_download`), and
//! falls back to the legacy plain-FLAC fetch (`stream_url` + `legacy`
//! post-processing) on any CMAF failure. Updates DB status + emits
//! `CacheEvent`s throughout both paths.

mod legacy;
mod permit;
mod stream_url;

use crate::event::{CacheEvent, CacheEventSink, CacheFormat};
use crate::types::OfflineCacheStatus;

use super::cmaf_path::try_cmaf_offline_download;

/// Used by both v2_cache_track_for_offline (single) and v2_cache_tracks_batch_for_offline (batch).
#[allow(clippy::too_many_arguments)]
pub fn spawn_track_cache_download(
    track_id: u64,
    file_path: std::path::PathBuf,
    client: std::sync::Arc<tokio::sync::RwLock<Option<qbz_qobuz::QobuzClient>>>,
    fetcher: std::sync::Arc<crate::StreamFetcher>,
    db: std::sync::Arc<tokio::sync::Mutex<Option<crate::OfflineCacheDb>>>,
    offline_root: String,
    library_db: std::sync::Arc<tokio::sync::Mutex<Option<qbz_library::LibraryDatabase>>>,
    sink: CacheEventSink,
    semaphore: std::sync::Arc<tokio::sync::Semaphore>,
) {
    tokio::spawn(async move {
        let Some(_permit) = permit::acquire_or_report(track_id, &db, &sink, &semaphore).await
        else {
            return;
        };

        if let Some(db_guard) = db.lock().await.as_ref() {
            let _ = db_guard.update_status(track_id, crate::OfflineCacheStatus::Downloading, None);
        }
        sink(CacheEvent::Started { track_id });

        // === CMAF-first offline download (v2 format) ===
        //
        // Stores bit-identical encrypted segments + wrapped content key.
        // Falls through to the legacy path below if any step fails (no
        // CoreBridge yet, /file/url returns a non-CMAF response, network
        // flake, vault init failure, etc.). The legacy fallback keeps
        // existing users unblocked while we validate the new path.
        match try_cmaf_offline_download(track_id, &db, &offline_root, &library_db, &client, &sink)
            .await
        {
            Ok(()) => return,
            Err(e) => {
                log::warn!(
                    "[Offline/CMAF] Track {} — CMAF path failed ({}), falling back to legacy /track/getFileUrl",
                    track_id,
                    e
                );
            }
        }

        let url = match stream_url::resolve_legacy_stream_url(&client, track_id).await {
            Ok(url) => url,
            Err(e) => {
                log::error!("Failed to get stream URL for track {}: {}", track_id, e);
                if let Some(db_guard) = db.lock().await.as_ref() {
                    let _ = db_guard.update_status(
                        track_id,
                        OfflineCacheStatus::Failed,
                        Some(&format!("Failed to get stream URL: {}", e)),
                    );
                }
                sink(CacheEvent::Failed { track_id, error: e });
                return;
            }
        };

        match fetcher
            .fetch_to_file(&url, &file_path, track_id, Some(&sink))
            .await
        {
            Ok(size) => {
                log::info!("Caching complete for track {}: {} bytes", track_id, size);
                if let Some(db_guard) = db.lock().await.as_ref() {
                    let _ = db_guard.mark_complete(track_id, size);
                }
                sink(CacheEvent::Completed {
                    track_id,
                    size,
                    format: CacheFormat::Flac,
                });

                legacy::post_process(
                    track_id,
                    &file_path,
                    &client,
                    &library_db,
                    &db,
                    &offline_root,
                    &sink,
                )
                .await;
            }
            Err(e) => {
                log::error!("Caching failed for track {}: {}", track_id, e);
                if let Some(db_guard) = db.lock().await.as_ref() {
                    let _ = db_guard.update_status(track_id, OfflineCacheStatus::Failed, Some(&e));
                }
                sink(CacheEvent::Failed { track_id, error: e });
            }
        }
    });
}
