//! The "fetch bytes -> Player::play_track" step of the engine, split out of
//! `engine/mod.rs` to keep both files under the line budget.

use slint::ComponentHandle;

use super::super::advance::{
    auto_skip_unavailable, is_forbidden_backoff, is_terminal_unavailable,
};
use super::super::loading::{clear_loading, set_loading};
use super::super::local::files::play_local_file_audible;
use super::super::quality::{local_playback_quality, set_viz_paused};
use super::super::state::PENDING_PLAY_ID;
use super::super::Runtime;
use super::offline_gate::offline_fast_fail_refused;
use crate::{AppWindow, NowPlayingState};

/// Run the audible step for `track_id`: grab the Qobuz client and call
/// the player's self-contained `play_track`. Errors are logged, not
/// surfaced — the poll loop keeps the UI consistent regardless.
pub(in super::super) async fn play_audible(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    track_id: u64,
) {
    // Offline fast-fail (slice 3d): refuse unplayable tracks BEFORE the
    // spinner/fetch. Every explicit play path (album/track/playlist/radio)
    // funnels through here after moving the queue cursor; the advance walks
    // pre-filter via `advance_to_playable`, so a refusal here means the user
    // explicitly picked an unavailable track.
    if offline_fast_fail_refused(runtime, weak, track_id).await {
        return;
    }
    // Raise the fetch spinner the instant playback is requested — BEFORE the
    // resolve/download/buffer below. The bar
    // already adopted the new track meta in `refresh_now_playing_meta`; this
    // bridges the silent gap until the poll loop sees the audio advancing.
    set_loading(weak, track_id);
    // Source-aware: a LOCAL user file plays from disk via the play_data seam.
    // Offline-cached + Qobuz keep the existing tier-walk below (unchanged), so
    // streaming playback can't regress. The current queue track tells us which
    // path to take via its `source`; the id guard avoids mis-routing when the
    // current track and `track_id` momentarily disagree. Auto-advance, skip and
    // play-all all flow through here, so they become source-aware for free.
    if let Some(qt) = runtime.core().current_track().await {
        if qt.id == track_id {
            match qt.source.as_deref() {
                Some("local") | Some("ephemeral") => {
                    play_local_file_audible(runtime, weak, track_id).await;
                    return;
                }
                _ => {}
            }
        }
    }
    // Offline-cached copy (preferred, decrypted to FLAC + played via play_data)
    // -> player L1/L2 -> Qobuz network. The offline handle is None before
    // login. The sink drives the padlock while a CMAF bundle decrypts.
    let offline = crate::offline::get().await;
    let sink = crate::offline_cache::row_sink(weak.clone());
    // Session resume: if this is the track restored at launch, start it at the
    // saved position (consumed once); any other track starts from 0.
    let start_position_secs = crate::session_persist::take_resume_for(track_id);
    match runtime
        .core()
        .play_track_resolved(
            track_id,
            // LOCAL playback: the device cap applies (#638 fix 3). The cast
            // branch above returned already, so a cast can never reach this.
            local_playback_quality().0,
            offline.as_deref(),
            Some(&sink),
            start_position_secs,
        )
        .await
    {
        Ok(()) => {
            // Player also cancels superseded play_track work; this gates any
            // post-success UI side effects if another play already owns the spinner.
            let still_current =
                PENDING_PLAY_ID.load(std::sync::atomic::Ordering::Relaxed) == track_id;
            if !still_current {
                log::info!(
                    "[qbz-slint] playback: play_track {track_id} completed but was superseded"
                );
            }
        }
        Err(e) => {
            log::error!("[qbz-slint] playback: play_track {track_id} failed: {e}");
            // Superseded fetch: the user already started another play while this
            // one was resolving. That newer play owns the cursor; do NOT skip.
            let still_current =
                PENDING_PLAY_ID.load(std::sync::atomic::Ordering::Relaxed) == track_id;
            // The fetch failed: no audio will advance, so the poll loop would never
            // clear the spinner. Drop it now (only if this play is still current).
            clear_loading(weak, track_id);
            // Tauri-parity regression fix: an unavailable track used to be
            // auto-skipped by the frontend (playbackService `autoSkipToNext`,
            // bounded, issue #467). Without this the queue cursor parks on the
            // dead track and playback stops. Terminal errors only: transient
            // failures were already retried by the client and should not skip.
            if still_current && is_forbidden_backoff(&e) {
                // Qobuz is 403'ing (or the client breaker is backing off).
                // Don't skip — it's not the track. Stop cleanly and surface it.
                log::warn!(
                    "[qbz-slint] playback: Qobuz 403 / backing off — not skipping (account or edge issue, see #637)"
                );
                crate::toast::show_weak(
                    weak,
                    qbz_i18n::t(
                        "Qobuz is temporarily refusing requests — backing off. Try again shortly.",
                    ),
                    crate::ToastKind::Error,
                );
                set_viz_paused(runtime, true);
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<NowPlayingState>().set_playing(false);
                });
            } else if still_current && is_terminal_unavailable(&e) {
                auto_skip_unavailable(runtime, weak, track_id).await;
            }
        }
    }
}
