//! "Add to offline cache" trigger for a single track.

use crate::adapter::SlintAdapter;
use crate::AppWindow;
use qbz_app::shell::AppRuntime;

use super::info::track_cache_info;
use super::sink::{push_status, row_sink};

pub(crate) type Runtime = std::sync::Arc<AppRuntime<SlintAdapter>>;

/// Cache a single track for offline playback. Fetches the track metadata,
/// pre-flights the cache limit, inserts the queued row, and spawns the
/// download (CMAF-first) with a row-updating sink.
pub fn cache_track(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    id: u64,
) {
    handle.spawn(async move {
        let Some(off) = crate::offline::get().await else {
            crate::toast::error_weak(&weak, qbz_i18n::t("Log in to cache tracks offline"));
            return;
        };
        let track = match runtime.core().get_track(id).await {
            Ok(t) => t,
            Err(e) => {
                log::error!("[qbz-slint] cache: get_track {id} failed: {e}");
                crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't load that track"));
                return;
            }
        };
        let info = track_cache_info(&track);
        let file_path = off.track_file_path(id, "flac");
        let file_path_str = file_path.to_string_lossy().to_string();

        // Pre-flight the cache limit, then insert the queued row.
        {
            let limit = *off.limit_bytes.lock().await;
            let guard = off.db.lock().await;
            let Some(db) = guard.as_ref() else {
                return;
            };
            let root = std::path::PathBuf::from(off.get_cache_path());
            if let Err(e) = qbz_offline_cache::maintenance::check_cache_limit(db, &root, limit) {
                log::warn!("[qbz-slint] cache limit reached: {e}");
                crate::toast::error_weak(
                    &weak,
                    qbz_i18n::t("Offline cache is full — free space or raise the limit"),
                );
                return;
            }
            if let Err(e) = db.insert_track(&info, &file_path_str) {
                log::error!("[qbz-slint] cache insert {id} failed: {e}");
                return;
            }
        }

        // Mark the row queued immediately, then spawn the download.
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
    });
}
