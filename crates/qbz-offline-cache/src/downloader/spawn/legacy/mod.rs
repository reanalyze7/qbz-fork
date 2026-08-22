//! Legacy-path post-processing after a successful plain-FLAC download:
//! tag write → artwork embed → organize file → cover.jpg save → bit-depth
//! detect → library insert → `Processed` event.
//!
//! This is a long sequential chain where each step logs-and-continues on
//! error rather than aborting — "best effort, never blocks the
//! already-cached file" — except metadata fetch and file organizing, whose
//! failure aborts the whole chain (there's nothing useful left to do
//! without them). Preserve this exact control flow; do not introduce early
//! returns beyond the two that already exist in `tag_and_organize`.

mod finalize;
mod tag_and_organize;

use crate::event::CacheEventSink;

#[allow(clippy::too_many_arguments)]
pub(super) async fn post_process(
    track_id: u64,
    file_path: &std::path::Path,
    client: &std::sync::Arc<tokio::sync::RwLock<Option<qbz_qobuz::QobuzClient>>>,
    library_db: &std::sync::Arc<tokio::sync::Mutex<Option<qbz_library::LibraryDatabase>>>,
    db: &std::sync::Arc<tokio::sync::Mutex<Option<crate::OfflineCacheDb>>>,
    offline_root: &str,
    sink: &CacheEventSink,
) {
    let Some((new_path, metadata)) =
        tag_and_organize::run(track_id, file_path, client, offline_root).await
    else {
        return;
    };

    finalize::run(track_id, &new_path, &metadata, library_db, db, sink).await;
}
