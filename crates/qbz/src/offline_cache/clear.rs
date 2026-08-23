//! Open the offline-cache folder + clear the entire cache.

use crate::AppWindow;

use super::ids::clear_cached_ids;

/// Open the offline-cache folder in the system file manager.
pub fn open_folder(handle: tokio::runtime::Handle) {
    handle.spawn(async move {
        let Some(off) = crate::offline::get().await else {
            return;
        };
        let path = off.get_cache_path();
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        if let Err(e) = std::process::Command::new(opener).arg(&path).spawn() {
            log::warn!("[qbz-slint] open offline folder failed: {e}");
        }
    });
}

/// Clear the entire offline cache (DB + on-disk bundles + library rows).
pub fn clear_all(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    handle.spawn(async move {
        let Some(off) = crate::offline::get().await else {
            return;
        };
        if let Err(e) = qbz_offline_cache::purge_all_cached_files(&off, &off.library_db).await {
            log::error!("[qbz-slint] clear offline cache failed: {e}");
            crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't clear the cache"));
            return;
        }
        clear_cached_ids();
        crate::toast::success_weak(&weak, qbz_i18n::t("Cache cleared"));
        crate::offline_manager::rebuild(weak.clone()).await;
    });
}
