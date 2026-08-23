//! Re-download triggers for a single track or a whole album's tracks.

use qbz_offline_cache::OfflineCacheStatus;

use crate::AppWindow;

use super::cache_single::Runtime;
use super::sink::{push_status, row_sink};

/// Re-download a single track (reset its row, spawn the download). Skips
/// in-flight downloads.
pub fn redownload_track(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    id: u64,
) {
    handle.spawn(async move {
        let Some(off) = crate::offline::get().await else {
            return;
        };
        {
            let guard = off.db.lock().await;
            let Some(db) = guard.as_ref() else {
                return;
            };
            if let Ok(Some(t)) = db.get_track(id) {
                if matches!(t.status, OfflineCacheStatus::Downloading) {
                    return;
                }
            }
            let _ = db.reset_track_for_redownload(id);
        }
        let file_path = off.track_file_path(id, "flac");
        push_status(&weak, id, 1, 0.0);
        qbz_offline_cache::spawn_track_cache_download(
            id,
            file_path,
            runtime.core().client(),
            off.fetcher.clone(),
            off.db.clone(),
            off.get_cache_path(),
            off.library_db.clone(),
            row_sink(weak.clone()),
            off.cache_semaphore.clone(),
        );
        crate::offline_manager::rebuild(weak.clone()).await;
    });
}

/// Re-download an album's tracks. `failed_only` re-queues only the failed
/// ones; otherwise all (skipping in-flight).
pub fn redownload_album(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    album_id: String,
    failed_only: bool,
) {
    handle.spawn(async move {
        let Some(off) = crate::offline::get().await else {
            return;
        };
        let targets: Vec<u64> = {
            let guard = off.db.lock().await;
            let Some(db) = guard.as_ref() else {
                return;
            };
            match db.get_album_tracks(&album_id) {
                Ok(tracks) => {
                    let picked =
                        qbz_offline_cache::maintenance::select_redownload_targets(&tracks, failed_only);
                    let ids: Vec<u64> = picked.iter().map(|t| t.track_id).collect();
                    for id in &ids {
                        let _ = db.reset_track_for_redownload(*id);
                    }
                    ids
                }
                Err(_) => Vec::new(),
            }
        };
        for id in targets {
            let file_path = off.track_file_path(id, "flac");
            push_status(&weak, id, 1, 0.0);
            qbz_offline_cache::spawn_track_cache_download(
                id,
                file_path,
                runtime.core().client(),
                off.fetcher.clone(),
                off.db.clone(),
                off.get_cache_path(),
                off.library_db.clone(),
                row_sink(weak.clone()),
                off.cache_semaphore.clone(),
            );
        }
        crate::offline_manager::rebuild(weak.clone()).await;
    });
}
