//! Pure filter for which cached tracks qualify as re-download targets.

use crate::{CachedTrackInfo, OfflineCacheStatus};

/// Filters tracks targeted by re-download: skip in-flight Downloading,
/// optionally restrict to Failed only.
pub fn select_redownload_targets(
    tracks: &[CachedTrackInfo],
    failed_only: bool,
) -> Vec<&CachedTrackInfo> {
    tracks
        .iter()
        .filter(|track| match track.status {
            OfflineCacheStatus::Downloading => false,
            OfflineCacheStatus::Failed => true,
            _ => !failed_only,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OfflineCacheStatus;

    fn track_with_status(id: u64, status: OfflineCacheStatus) -> CachedTrackInfo {
        CachedTrackInfo {
            track_id: id,
            title: format!("t{}", id),
            artist: "A".into(),
            album: None,
            album_id: None,
            duration_secs: 0,
            file_size_bytes: 0,
            quality: "lossless".into(),
            bit_depth: None,
            sample_rate: None,
            status,
            progress_percent: 0,
            error_message: None,
            created_at: "".into(),
            last_accessed_at: "".into(),
            artwork_path: None,
            file_path: "".into(),
        }
    }

    #[test]
    fn redownload_targets_full_skips_only_downloading() {
        let tracks = vec![
            track_with_status(1, OfflineCacheStatus::Ready),
            track_with_status(2, OfflineCacheStatus::Downloading),
            track_with_status(3, OfflineCacheStatus::Failed),
            track_with_status(4, OfflineCacheStatus::Queued),
        ];
        let picked = select_redownload_targets(&tracks, false);
        let ids: Vec<u64> = picked.iter().map(|track| track.track_id).collect();
        assert_eq!(ids, vec![1, 3, 4]);
    }

    #[test]
    fn redownload_targets_failed_only_returns_failed() {
        let tracks = vec![
            track_with_status(1, OfflineCacheStatus::Ready),
            track_with_status(2, OfflineCacheStatus::Failed),
            track_with_status(3, OfflineCacheStatus::Downloading),
        ];
        let picked = select_redownload_targets(&tracks, true);
        let ids: Vec<u64> = picked.iter().map(|track| track.track_id).collect();
        assert_eq!(ids, vec![2]);
    }
}
