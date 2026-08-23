//! Per-user runtime start: capture the tokio handle, seed ListenBrainz
//! credentials from the shared cache, and start the offline-queue flush
//! watcher.

use std::sync::OnceLock;

use qbz_integrations::listenbrainz::cache::ListenBrainzCache;

use crate::scrobbler_settings;

use super::flush::flush_offline_queues;
use super::{listenbrainz_cache_path, RT_HANDLE};

/// One-shot guard for the engine-watch flush task (lives for the process).
static FLUSH_WATCHER: OnceLock<()> = OnceLock::new();

/// Per-user runtime start, called from `init_shell_for_user` AFTER
/// `scrobbler_settings::init_for_user`. Captures the tokio handle for the
/// fire path, seeds ListenBrainz credentials from the SHARED cache when this
/// build has none (a Tauri sign-in carries over), starts the offline-engine
/// flush watcher (once per process), and kicks an initial queue flush.
pub fn start(handle: tokio::runtime::Handle) {
    if let Ok(mut g) = RT_HANDLE.lock() {
        *g = Some(handle.clone());
    }

    handle.spawn(async move {
        seed_listenbrainz_from_shared_cache().await;
        if !crate::offline_mode::engine().is_offline() {
            flush_offline_queues().await;
        }
    });

    let watcher_handle = handle.clone();
    FLUSH_WATCHER.get_or_init(move || {
        watcher_handle.spawn(async move {
            let mut rx = crate::offline_mode::engine().subscribe();
            let mut was_offline = rx.borrow_and_update().is_offline();
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                let offline = rx.borrow_and_update().is_offline();
                if was_offline && !offline {
                    log::info!("[qbz-slint] scrobblers: back online, flushing queues");
                    flush_offline_queues().await;
                }
                was_offline = offline;
            }
        });
    });
}

/// Adopt the shared `ListenBrainzCache` credentials (the row the Tauri build
/// persists to) when this build's store has no LB token yet. Enable flags are
/// NOT touched — scrobbling stays opt-in per build.
async fn seed_listenbrainz_from_shared_cache() {
    if scrobbler_settings::get().listenbrainz_is_authed() {
        return;
    }
    let Some(path) = listenbrainz_cache_path() else {
        return;
    };
    let creds = tokio::task::spawn_blocking(move || {
        ListenBrainzCache::new(&path).and_then(|c| c.get_credentials())
    })
    .await;
    if let Ok(Ok((Some(token), Some(user_name)))) = creds {
        if !token.is_empty() {
            log::info!("[qbz-slint] adopting ListenBrainz credentials from shared cache");
            scrobbler_settings::set_listenbrainz_token(&token, &user_name);
        }
    }
}
