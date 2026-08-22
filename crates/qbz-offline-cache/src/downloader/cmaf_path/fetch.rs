//! Step 1: fetch the raw CMAF bundle over the network, with a progress
//! callback wired to the shared `CacheEventSink`.

use crate::event::{CacheEvent, CacheEventSink};
use qbz_models::Quality;
use qbz_qobuz::cmaf::CmafRawBundle;

/// Fetch the raw CMAF bundle. Requires an initialized QobuzClient; if
/// it is missing, bail so the legacy path runs.
pub(super) async fn fetch_raw_bundle(
    track_id: u64,
    client: &std::sync::Arc<tokio::sync::RwLock<Option<qbz_qobuz::QobuzClient>>>,
    sink: &CacheEventSink,
) -> Result<CmafRawBundle, String> {
    let sink_for_cb = sink.clone();
    let progress_cb: qbz_qobuz::CmafProgressCallback = std::sync::Arc::new(
        move |update: qbz_qobuz::CmafProgressUpdate| {
            let percent = if update.n_segments > 0 {
                (update.segments_completed as f64 / update.n_segments as f64 * 100.0)
                    .round()
                    .clamp(0.0, 100.0) as u8
            } else {
                0u8
            };
            sink_for_cb(CacheEvent::Progress {
                track_id,
                progress_percent: percent,
                bytes_downloaded: update.bytes_this_segment,
                total_bytes: None,
            });
        },
    );

    let client_guard = client.read().await;
    let qobuz_client = client_guard
        .as_ref()
        .ok_or_else(|| "QobuzClient not initialized".to_string())?;
    qbz_qobuz::cmaf::download_raw_with_progress(
        qobuz_client,
        track_id,
        Quality::UltraHiRes,
        Some(progress_cb),
    )
    .await
}
