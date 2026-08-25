//! Now-playing metadata sync: title/artist/album/artwork/quality-badge/
//! context/MPRIS/tray/notification, driven off the current queue track.
use slint::ComponentHandle;

mod artwork;
mod artwork_large;
mod fields;
mod fields_types;
mod hydrate;
mod mpris_tray;
mod push_ui;
mod quality_badge;
mod quality_fields;
mod record_recent;
mod statics;

pub(crate) use quality_badge::{classify_limit_cause, delivered_tier_str, stream_downgraded};
pub(super) use record_recent::record_recent;
pub use statics::NOTIFICATIONS_ENABLED;

use super::Runtime;
use crate::{AppWindow, NowPlayingState};
use qbz_models::RepeatMode;

/// Refresh the now-playing bar + MPRIS/tray/notification from the current
/// queue track. Re-runs on track change, resume/seek, and a mid-track
/// quality patch — the per-piece dedupe guards (MPRIS art, the desktop
/// notification, `FORCE_UI_REPUSH`) keep repeated calls cheap and correct.
pub(crate) async fn refresh_now_playing_meta(runtime: &Runtime, weak: &slint::Weak<AppWindow>) {
    let state = runtime.core().get_queue_state().await;
    // Seed the transport shuffle/repeat onto the bar so a restored (or already
    // shuffling/repeating) queue lights those buttons — previously only a manual
    // toggle set NowPlayingState.shuffle/repeat, so after a session restore the
    // state was ON in the core but the NPB shuffle/repeat icons stayed dark.
    // RepeatMode -> the i32 the NPB uses (mirrors cycle-repeat's mapping).
    let shuffle_seed = state.shuffle;
    let repeat_seed = match state.repeat {
        RepeatMode::Off => 0,
        RepeatMode::All => 1,
        RepeatMode::One => 2,
    };
    let Some(track) = state.current_track else {
        clear_now_playing(weak);
        return;
    };
    let built = fields::build_meta_fields(runtime, weak, &track);
    mpris_tray::sync_mpris_and_tray(weak, &built);
    push_ui::finish_meta_push(runtime, weak, shuffle_seed, repeat_seed, built);
}

/// No current track → clear the tray tooltip (Linux) + stop media controls,
/// and reset the notify/MPRIS dedupe guards so replaying the same track
/// after a stop fires them again.
fn clear_now_playing(weak: &slint::Weak<AppWindow>) {
    statics::NOTIFY_LAST_TRACK.store(u64::MAX, std::sync::atomic::Ordering::Relaxed);
    if let Ok(mut last) = statics::MPRIS_LAST_META.lock() {
        *last = None;
    }
    if let Some(t) = crate::tray::handle() {
        t.clear_track();
    }
    if let Some(mc) = crate::media_controls::handle() {
        mc.set_playback(qbz_media_controls::PlaybackStatus::Stopped, None);
    }
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<NowPlayingState>().set_has_track(false);
    });
}
