//! CMAF-first offline download path (v2 format).
//!
//! Split into this orchestrator (`try_cmaf_offline_download`) and its
//! sequential steps in sibling files: fetch the raw bundle (`fetch`), wrap
//! the keying material + persist to disk + flip the DB row (`persist`),
//! and populate the library row with metadata/artwork (`library_row`).

mod fetch;
mod library_row;
mod persist;

use crate::event::{CacheEvent, CacheEventSink, CacheFormat};

/// Shared helper: spawn the download task for a single track.
/// CMAF-first offline download path (v2 format).
///
/// On success: the encrypted CMAF bundle is persisted under
/// `<offline_root>/tracks-cmaf/<track_id>/`, the per-track AES content key
/// + session infos are wrapped via `qbz-secrets` and stored on the DB row,
/// `cache_format` flips to 2, `mark_complete` fires, and the library row
/// is populated with the same metadata the legacy path would populate
/// (title/artist/album, etc.).
///
/// Returns `Err` for any failure that makes CMAF unusable — the caller
/// falls back to the legacy plain-FLAC path.
pub(crate) async fn try_cmaf_offline_download(
    track_id: u64,
    db: &std::sync::Arc<tokio::sync::Mutex<Option<crate::OfflineCacheDb>>>,
    offline_root: &str,
    library_db: &std::sync::Arc<tokio::sync::Mutex<Option<qbz_library::LibraryDatabase>>>,
    client: &std::sync::Arc<tokio::sync::RwLock<Option<qbz_qobuz::QobuzClient>>>,
    sink: &CacheEventSink,
) -> Result<(), String> {
    let offline_root_path = std::path::PathBuf::from(offline_root);

    // Progress callback: emit the same `offline:caching_progress` event
    // shape the legacy StreamFetcher fires, so the UI's progress ring
    // doesn't care whether the bytes came from CMAF or legacy.
    //
    // Note: one 'started' event here up front so the frontend sees
    // 'downloading' status immediately (the ring starts empty); actual
    // percentage updates arrive per completed segment.
    sink(CacheEvent::Progress {
        track_id,
        progress_percent: 0,
        bytes_downloaded: 0,
        total_bytes: None,
    });

    let bundle = fetch::fetch_raw_bundle(track_id, client, sink).await?;

    let (layout, total_bytes) =
        persist::wrap_persist_and_record(track_id, db, &offline_root_path, &bundle).await?;

    library_row::populate_library_row(track_id, client, library_db, &layout, &bundle).await;

    log::info!(
        "[Offline/CMAF] Track {} cached as v2 bundle: {:.2} MB under {:?}",
        track_id,
        total_bytes as f64 / (1024.0 * 1024.0),
        layout.track_dir
    );
    sink(CacheEvent::Completed {
        track_id,
        size: total_bytes,
        format: CacheFormat::Cmaf,
    });
    sink(CacheEvent::Processed {
        track_id,
        path: layout.track_dir.to_string_lossy().to_string(),
        format: CacheFormat::Cmaf,
    });
    Ok(())
}
