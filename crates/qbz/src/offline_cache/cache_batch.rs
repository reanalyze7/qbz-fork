//! "Add to offline cache" trigger for a batch of already-fetched tracks.

use crate::AppWindow;

use super::cache_single::Runtime;
use super::info::track_cache_info;
use super::sink::{push_status, row_sink};

/// Cache a batch of already-fetched catalog tracks (favorites bulk action).
pub fn cache_tracks(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    tracks: Vec<qbz_models::Track>,
) {
    if tracks.is_empty() {
        return;
    }
    handle.spawn(async move {
        let Some(off) = crate::offline::get().await else {
            crate::toast::error_weak(&weak, qbz_i18n::t("Log in to cache tracks offline"));
            return;
        };
        // Pre-flight once for the whole batch (mirrors Tauri).
        {
            let limit = *off.limit_bytes.lock().await;
            let guard = off.db.lock().await;
            let Some(db) = guard.as_ref() else {
                return;
            };
            let root = std::path::PathBuf::from(off.get_cache_path());
            if let Err(e) = qbz_offline_cache::maintenance::check_cache_limit(db, &root, limit) {
                log::warn!("[qbz-slint] batch cache limit reached: {e}");
                crate::toast::error_weak(
                    &weak,
                    qbz_i18n::t("Offline cache is full — free space or raise the limit"),
                );
                return;
            }
        }
        let count = tracks.len();
        for track in &tracks {
            let id = track.id;
            let info = track_cache_info(track);
            let file_path = off.track_file_path(id, "flac");
            let file_path_str = file_path.to_string_lossy().to_string();
            {
                let guard = off.db.lock().await;
                let Some(db) = guard.as_ref() else {
                    return;
                };
                if db.insert_track(&info, &file_path_str).is_err() {
                    continue;
                }
            }
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
        crate::toast::success_weak(
            &weak,
            qbz_i18n::tf(
                "Caching {} track offline…",
                "Caching {} tracks offline…",
                count as i64,
                &[&count.to_string()],
            ),
        );
    });
}
