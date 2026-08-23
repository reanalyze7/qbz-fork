//! D4 activation-time cache purge consumer.

use super::subscription::now_unix_secs;
use super::SUBSCRIPTION;

/// Mirror of Tauri's activation-time purge consumer
/// (`session_lifecycle.rs` `activate_session`, lines ~237-264): when the
/// subscription has been invalid past the grace window, purge the offline
/// cache once and record the purge. Runs detached so session entry never
/// blocks on it.
///
/// Init-order dependency: this expects `crate::offline::activate` to have
/// already run before `offline_mode::init_for_user` calls this — do not
/// reorder those two calls.
pub(super) fn spawn_subscription_purge_check() {
    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        log::warn!("[qbz-slint] subscription purge check: no tokio runtime, skipped");
        return;
    };
    handle.spawn(async move {
        let now = now_unix_secs();
        // Read the verdict without holding the lock across awaits.
        let should_purge = SUBSCRIPTION
            .lock()
            .ok()
            .and_then(|guard| {
                guard
                    .as_ref()
                    .and_then(|store| store.should_purge_offline_cache(now).ok())
            })
            .unwrap_or(false);
        if !should_purge {
            return;
        }

        log::warn!(
            "[qbz-slint] Subscription invalid beyond the grace window. Purging offline cache."
        );
        let Some(off) = crate::offline::get().await else {
            // init order puts offline::activate before init_for_user; this
            // only triggers if that ordering regresses. Re-checked next
            // activation, so the purge is deferred, not lost.
            log::warn!("[qbz-slint] purge deferred: offline cache not active");
            return;
        };
        if let Err(e) = qbz_offline_cache::purge_all_cached_files(&off, &off.library_db).await {
            log::error!("[qbz-slint] failed to purge offline cache: {e}");
            return;
        }
        // Resync the in-memory cached-ids set the track rows read.
        crate::offline_cache::load_cached_ids().await;
        if let Ok(guard) = SUBSCRIPTION.lock() {
            if let Some(store) = guard.as_ref() {
                let _ = store.mark_offline_cache_purged(now);
            }
        }
        log::info!("[qbz-slint] offline cache purged (subscription grace elapsed)");
    });
}
