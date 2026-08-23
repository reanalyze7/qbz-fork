//! Offline fast-fail gate for `play_audible`, split out to keep that file
//! under the line budget.

use super::super::advance::{offline_playability, OfflinePlayability};
use super::super::Runtime;
use crate::AppWindow;

/// Offline fast-fail (slice 3d): refuse unplayable tracks BEFORE the
/// spinner/fetch. Every explicit play path (album/track/playlist/radio)
/// funnels through here after moving the queue cursor; the advance walks
/// pre-filter via `advance_to_playable`, so a refusal here means the user
/// explicitly picked an unavailable track. Returns `true` when the play was
/// refused (caller should return immediately without starting the spinner).
pub(super) async fn offline_fast_fail_refused(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    track_id: u64,
) -> bool {
    if !crate::offline_mode::engine().is_offline() {
        return false;
    }
    let Some(qt) = runtime.core().current_track().await else {
        return false;
    };
    if qt.id != track_id {
        return false;
    }
    match offline_playability(&qt) {
        OfflinePlayability::Playable => false,
        OfflinePlayability::GraceExpired => {
            log::info!(
                "[qbz-slint] offline: refused track {track_id} (subscription grace expired)"
            );
            crate::toast::show_weak(
                weak,
                qbz_i18n::t(
                    "Offline listening period expired — reconnect to verify your subscription",
                ),
                crate::ToastKind::Warning,
            );
            true
        }
        OfflinePlayability::Unavailable => {
            log::info!("[qbz-slint] offline: refused track {track_id} (not available offline)");
            crate::toast::show_weak(
                weak,
                qbz_i18n::t("Track not available offline"),
                crate::ToastKind::Warning,
            );
            true
        }
        OfflinePlayability::FileMissing => {
            log::info!(
                "[qbz-slint] local play: refused track {track_id} (file missing — unmounted drive?)"
            );
            crate::toast::show_weak(
                weak,
                qbz_i18n::t("File not available — is the drive mounted?"),
                crate::ToastKind::Warning,
            );
            true
        }
    }
}
