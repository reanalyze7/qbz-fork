//! Async wrapper that emits `UnlockStart`/`UnlockEnd` around the blocking
//! CMAF resolution, so the frontend can show an "unlocking" animation.

use std::path::Path;

use crate::db::CmafBundleRow;
use crate::event::{CacheEvent, CacheEventSink};

use super::resolve::load_cmaf_bundle;

/// Run `load_cmaf_bundle` on the blocking pool and emit `UnlockStart` /
/// `UnlockEnd` through the sink so the frontend can show an "unlocking"
/// animation on the track row.
///
/// `display_track_id` is what the frontend knows this track as — for the
/// Qobuz flow it's the Qobuz track id, for Local Library it's the library
/// row id. The events carry THIS id so the UI can key off it.
///
/// `cmaf_track_id` is the key `load_cmaf_bundle` logs against (always the
/// Qobuz track id — that's what the bundle is identified by on disk).
pub async fn load_cmaf_bundle_with_ui_events(
    sink: &CacheEventSink,
    display_track_id: u64,
    cmaf_track_id: u64,
    row: CmafBundleRow,
    cache_path: String,
) -> Option<Vec<u8>> {
    sink(CacheEvent::UnlockStart {
        track_id: display_track_id,
    });
    let result = tokio::task::spawn_blocking(move || {
        load_cmaf_bundle(cmaf_track_id, &row, Path::new(&cache_path))
    })
    .await
    .ok()
    .flatten();
    sink(CacheEvent::UnlockEnd {
        track_id: display_track_id,
        success: result.is_some(),
    });
    result
}
