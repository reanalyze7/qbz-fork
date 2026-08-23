//! Remove / refresh a single track's offline copy.

use crate::AppWindow;

use super::cache_single::{cache_track, Runtime};
use super::ids::mark_cached;
use super::sink::push_status;

/// Remove a track's offline copy (DB row + on-disk bundle/file + library row).
pub fn remove_cached(
    _runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    id: u64,
) {
    handle.spawn(async move {
        remove_cached_inner(&weak, id, true).await;
    });
}

/// Refresh a track's offline copy ("Refresh offline copy", the cached-state
/// menu entry — Tauri's ready-submenu re-download): drop the cached copy,
/// then re-queue the download. Sequenced in ONE task — `cache_track`'s
/// `insert_track` is not an upsert, so the delete must land first.
pub fn refresh_cached(
    runtime: Runtime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    id: u64,
) {
    let handle2 = handle.clone();
    handle.spawn(async move {
        // No "Removed from offline" toast mid-refresh; cache_track's own
        // status pushes take over from here.
        remove_cached_inner(&weak, id, false).await;
        cache_track(runtime, weak, handle2, id);
    });
}

/// The removal body shared by [`remove_cached`] and [`refresh_cached`].
pub(super) async fn remove_cached_inner(weak: &slint::Weak<AppWindow>, id: u64, toast: bool) {
    let Some(off) = crate::offline::get().await else {
        return;
    };
    let removed_path = {
        let guard = off.db.lock().await;
        match guard.as_ref() {
            Some(db) => db.delete_track(id).ok().flatten(),
            None => return,
        }
    };
    if let Some(p) = removed_path {
        let path = std::path::Path::new(&p);
        // v2 bundles live in `tracks-cmaf/<id>/` — remove the whole dir.
        let looks_v2 = path
            .parent()
            .and_then(|pp| pp.parent())
            .and_then(|r| r.file_name())
            .and_then(|n| n.to_str())
            == Some("tracks-cmaf");
        if looks_v2 {
            if let Some(dir) = path.parent() {
                let _ = std::fs::remove_dir_all(dir);
            }
        } else {
            let _ = std::fs::remove_file(path);
        }
    }
    {
        let guard = off.library_db.lock().await;
        if let Some(db) = guard.as_ref() {
            let _ = db.remove_qobuz_cached_track(id);
        }
    }
    mark_cached(id, false);
    push_status(weak, id, 0, 0.0);
    if toast {
        crate::toast::success_weak(weak, qbz_i18n::t("Removed from offline"));
    }
    crate::offline_manager::rebuild(weak.clone()).await;
}
