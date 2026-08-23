//! Per-user subscription state (D4): lifecycle binding + grace/purge verdicts.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use qbz_app::settings::subscription::SubscriptionStateStore;
use qbz_app::user_data::UserDataPaths;

use super::purge::spawn_subscription_purge_check;
use super::{engine, SUBSCRIPTION};

/// `<data_dir>/qbz/users/<user_id>/` — the per-user directory both the
/// engine store and the subscription store live in. Matches the Tauri
/// per-user path (and `tray_settings::user_dir`).
pub fn user_data_dir(user_id: u64) -> Option<PathBuf> {
    Some(
        dirs::data_dir()?
            .join("qbz")
            .join("users")
            .join(user_id.to_string()),
    )
}

/// Bind the engine + subscription store to the active user's data dir.
/// Called on every session activation (login, restore, offline entry),
/// AFTER `crate::offline::activate` so the purge consumer can reach the
/// offline cache. Best-effort: failures are logged, never block entry.
///
/// Must run within the tokio runtime context (the purge check spawns).
pub fn init_for_user(base_dir: &Path) {
    if let Err(e) = engine().init_for_user(base_dir) {
        log::error!("[qbz-slint] offline mode engine init failed: {e}");
    }
    match SubscriptionStateStore::new_at(base_dir) {
        Ok(store) => {
            if let Ok(mut guard) = SUBSCRIPTION.lock() {
                *guard = Some(store);
            }
        }
        Err(e) => log::error!("[qbz-slint] subscription state store open failed: {e}"), // fail-open
    }
    spawn_subscription_purge_check();
}

/// Drop the per-user state on logout. The engine also ends the
/// session-scoped offline state (offline_session + cached induced flag) and
/// reopens the Qobuz gate when connectivity allows — a logged-out user must
/// always be able to sign back in. The persisted induced preference reloads
/// from disk on the next session activation.
pub fn teardown() {
    engine().teardown();
    if let Ok(mut guard) = SUBSCRIPTION.lock() {
        *guard = None;
    }
}

pub(super) fn now_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// D4 producer: a successful login verdict. Clears any running grace clock.
pub fn subscription_mark_valid() {
    let now = now_unix_secs();
    match SUBSCRIPTION.lock() {
        Ok(guard) => match guard.as_ref() {
            Some(store) => {
                if let Err(e) = store.mark_valid(now) {
                    log::error!("[qbz-slint] subscription mark_valid failed: {e}");
                }
            }
            None => log::warn!("[qbz-slint] subscription mark_valid: no store open"),
        },
        Err(e) => log::error!("[qbz-slint] subscription store lock poisoned: {e}"),
    }
}

/// D4 producer: an EXPLICIT ineligible-account login verdict
/// (`ApiError::IneligibleUser`). Generic 401/network errors must never
/// reach this — the grace clock only starts on a real verdict.
///
/// An ineligible verdict can arrive before any session activation (the
/// failed login never activates), so when no store is open this falls back
/// to transiently opening the LAST user's store.
pub fn subscription_mark_invalid() {
    let now = now_unix_secs();
    if let Ok(guard) = SUBSCRIPTION.lock() {
        if let Some(store) = guard.as_ref() {
            if let Err(e) = store.mark_invalid(now) {
                log::error!("[qbz-slint] subscription mark_invalid failed: {e}");
            }
            return;
        }
    }
    let Some(user_id) = UserDataPaths::load_last_user_id() else {
        log::warn!("[qbz-slint] subscription mark_invalid: no previous user, skipping");
        return;
    };
    let Some(dir) = user_data_dir(user_id) else {
        log::warn!("[qbz-slint] subscription mark_invalid: data dir unavailable");
        return;
    };
    match SubscriptionStateStore::new_at(&dir) {
        Ok(store) => {
            if let Err(e) = store.mark_invalid(now) {
                log::error!("[qbz-slint] subscription mark_invalid failed: {e}");
            }
        }
        Err(e) => log::error!("[qbz-slint] subscription state store open failed: {e}"),
    }
}

/// D4 consumer: may the offline cache serve FULL tracks right now? Binary —
/// within the 30-day grace window yes, past it no; there is NO 30-second
/// preview path. Fail-open `true` when no store is bound. Consumed by the
/// playback gating (`playback::offline_playability`).
pub fn offline_playback_allowed() -> bool {
    let now = now_unix_secs();
    SUBSCRIPTION
        .lock()
        .ok()
        .and_then(|guard| {
            guard
                .as_ref()
                .map(|store| store.offline_playback_allowed(now).unwrap_or(true))
        })
        .unwrap_or(true)
}
