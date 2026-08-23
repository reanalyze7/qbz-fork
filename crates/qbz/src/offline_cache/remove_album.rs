//! Remove a whole album's offline copies.

use crate::AppWindow;

use super::ids::mark_cached;

/// Remove a whole album's offline copies (rows + CMAF dirs + library rows).
pub fn remove_album(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    album_id: String,
) {
    handle.spawn(async move {
        let Some(off) = crate::offline::get().await else {
            return;
        };
        let report = {
            let guard = off.db.lock().await;
            let Some(db) = guard.as_ref() else {
                return;
            };
            let root = std::path::PathBuf::from(off.get_cache_path());
            qbz_offline_cache::maintenance::remove_album_cached_tracks(db, &root, &album_id)
        };
        let report = match report {
            Ok(r) => r,
            Err(e) => {
                log::error!("[qbz-slint] remove album {album_id} failed: {e}");
                return;
            }
        };
        {
            let guard = off.library_db.lock().await;
            if let Some(db) = guard.as_ref() {
                for id in &report.removed_track_ids {
                    let _ = db.remove_qobuz_cached_track(*id);
                }
            }
        }
        for id in &report.removed_track_ids {
            mark_cached(*id, false);
        }
        let ids = report.removed_track_ids.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            for id in ids {
                crate::set_row_cache_status(&w, &id.to_string(), 0, 0.0);
            }
        });
        crate::toast::success_weak(&weak, qbz_i18n::t("Removed album from offline"));
        crate::offline_manager::rebuild(weak.clone()).await;
    });
}
