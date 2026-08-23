//! Formatting helpers shared by the rollup builder.

use qbz_offline_cache::{CachedTrackInfo, OfflineCacheStatus};

use super::GB;

pub(crate) fn human_size(bytes: u64) -> String {
    let b = bytes as f64;
    if bytes >= GB {
        format!("{:.1} GB", b / GB as f64)
    } else if bytes >= 1024 * 1024 {
        format!("{:.0} MB", b / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.0} KB", b / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

pub(super) fn track_status_int(s: &OfflineCacheStatus) -> i32 {
    match s {
        OfflineCacheStatus::Ready => 3,
        OfflineCacheStatus::Failed => 4,
        _ => 2,
    }
}

pub(super) fn fmt_duration(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub(super) fn album_size(group: &[CachedTrackInfo]) -> u64 {
    group.iter().map(|t| t.file_size_bytes).sum()
}

/// Path of an album's on-disk cover thumbnail, or "" when none exists.
/// Resolution order (B5): the index's `artwork_path` when set, the CMAF
/// bundle's `tracks-cmaf/<id>/cover.jpg`, then the `cover.jpg` sibling of
/// the audio file (v1-format rows). Computed on the worker (a `String` is
/// `Send`).
pub(super) fn cover_path(cache_path: &str, track: &CachedTrackInfo) -> String {
    track.resolve_cover_path(cache_path).unwrap_or_default()
}

/// Album rows render their cover at 40px; decode to the rows tier so the
/// model holds ~36KB per cover instead of the full-resolution source.
pub(super) const COVER_DECODE_SIZE: u32 = 96;
