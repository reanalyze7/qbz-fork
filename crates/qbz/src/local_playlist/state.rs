//! The open local playlist's playable queue snapshot + repo-position map.
//! Read and written from `detail_local`, `detail_offline_mixed`, `playback`,
//! `reorder`, and `remove` — every other module reaches these through
//! `use super::state::*` rather than re-declaring the statics.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use qbz_models::QueueTrack;

/// The open local playlist's playable queue snapshot, aligned with the row
/// `TrackItem.id`s (`QueueTrack.id.to_string()`), plus per-item repo
/// positions for removal. Mirrors `playlist.rs::CURRENT` for Qobuz lists.
pub(crate) static CURRENT_QUEUE: LazyLock<Mutex<Vec<QueueTrack>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
/// (playlist id, offline_only) of the open local playlist detail.
pub(crate) static CURRENT_META: LazyLock<Mutex<Option<(String, bool)>>> =
    LazyLock::new(|| Mutex::new(None));
/// Row TrackItem id -> repo `position` (for remove-selected).
pub(crate) static ROW_POSITIONS: LazyLock<Mutex<HashMap<String, i32>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The ready, SOURCE-AWARE QueueTrack of an open-detail row by display id
/// (any source — snapshot rows are built to enqueue as-is). `None` for
/// rows not in the open snapshot: unplayable rows (file:/broken:/
/// unresolved) and any id while no snapshot-backed detail is open. The
/// per-row / bulk Play next + Add to queue routing reads this (spec §3.2).
pub fn queue_track_for_row(id: &str) -> Option<QueueTrack> {
    let queue = CURRENT_QUEUE.lock().ok()?;
    queue.iter().find(|q| q.id.to_string() == id).cloned()
}

/// Local-mode picker ref for an open-detail row id: `"<library row id>"` for
/// local file rows. `None` for Qobuz/offline-copy rows (those ride the
/// catalog-id flow) and for ids not in the open snapshot.
pub fn local_picker_ref_for_row(id: &str) -> Option<String> {
    let queue = CURRENT_QUEUE.lock().ok()?;
    let q = queue.iter().find(|q| q.id.to_string() == id)?;
    match q.source.as_deref() {
        Some("local") => Some(q.id.to_string()),
        _ => None,
    }
}

/// Adopt the ONLINE mixed Qobuz detail's merged queue snapshot into the
/// open-detail statics this module owns (CURRENT_QUEUE / CURRENT_META /
/// ROW_POSITIONS), so `play_from_visible` / `play_all` /
/// `local_picker_ref_for_row` / drag work over the merged rows exactly like
/// the LOCAL and offline details (row identity E11). `offline_only` is
/// always false here — a real Qobuz playlist never stamps the D8 guard; the
/// QConnect queue-push exclusion of the local rows happens per-track at
/// admission (`QueueTrack.source`). UI thread.
pub fn set_open_mixed_snapshot(
    playlist_id: &str,
    queue: Vec<QueueTrack>,
    positions: HashMap<String, i32>,
) {
    if let Ok(mut cur) = CURRENT_QUEUE.lock() {
        *cur = queue;
    }
    if let Ok(mut meta) = CURRENT_META.lock() {
        *meta = Some((playlist_id.to_string(), false));
    }
    if let Ok(mut pos) = ROW_POSITIONS.lock() {
        *pos = positions;
    }
}

/// Clear the open-detail snapshot (pure-Qobuz detail / navigation reset) so
/// stale local rows from a previously open detail can never resolve.
pub fn clear_open_snapshot() {
    if let Ok(mut cur) = CURRENT_QUEUE.lock() {
        cur.clear();
    }
    if let Ok(mut meta) = CURRENT_META.lock() {
        *meta = None;
    }
    if let Ok(mut pos) = ROW_POSITIONS.lock() {
        pos.clear();
    }
}
