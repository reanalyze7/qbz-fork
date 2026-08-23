//! The `CacheEventSink` builder that reflects events onto visible rows.

use std::sync::Arc;

use qbz_offline_cache::{CacheEvent, CacheEventSink};

use crate::AppWindow;

use super::ids::mark_cached;

/// Build a sink that reflects cache + unlock events onto every visible row
/// matching the event's track id (and surfaces terminal toasts). Shared by
/// the cache trigger AND the play path (UnlockStart/End → padlock).
pub fn row_sink(weak: slint::Weak<AppWindow>) -> CacheEventSink {
    Arc::new(move |ev: CacheEvent| match ev {
        CacheEvent::Started { track_id } => {
            push_status(&weak, track_id, 2, 0.0);
        }
        CacheEvent::Progress {
            track_id,
            progress_percent,
            ..
        } => {
            let p = (progress_percent as f32 / 100.0).clamp(0.0, 1.0);
            push_status(&weak, track_id, 2, p);
        }
        CacheEvent::Completed { track_id, .. } => {
            mark_cached(track_id, true);
            push_status(&weak, track_id, 3, 1.0);
            crate::toast::success_weak(&weak, qbz_i18n::t("Cached for offline"));
        }
        CacheEvent::Processed { .. } => {
            // Post-processing done; status already 'ready' from Completed.
        }
        CacheEvent::Failed { track_id, error } => {
            log::warn!("[qbz-slint] offline cache failed for {track_id}: {error}");
            push_status(&weak, track_id, 4, 0.0);
            crate::toast::error_weak(&weak, qbz_i18n::t("Offline caching failed"));
        }
        CacheEvent::UnlockStart { track_id } => {
            push_unlocking(&weak, track_id, true);
        }
        CacheEvent::UnlockEnd { track_id, .. } => {
            push_unlocking(&weak, track_id, false);
        }
    })
}

pub(super) fn push_status(weak: &slint::Weak<AppWindow>, track_id: u64, status: i32, progress: f32) {
    let id = track_id.to_string();
    let _ = weak.upgrade_in_event_loop(move |w| {
        crate::set_row_cache_status(&w, &id, status, progress);
    });
}

pub(super) fn push_unlocking(weak: &slint::Weak<AppWindow>, track_id: u64, unlocking: bool) {
    let id = track_id.to_string();
    let _ = weak.upgrade_in_event_loop(move |w| {
        crate::set_row_unlocking(&w, &id, unlocking);
    });
}
