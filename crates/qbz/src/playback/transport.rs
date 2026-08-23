//! Transport controls: play/pause/next/previous/seek/volume/mute/shuffle/
//! repeat.

use super::advance::advance_to_playable;
use super::engine::after_track_change;
use super::quality::set_viz_paused;
use super::state::refresh_sidebar;
use super::Runtime;
use crate::AppWindow;

/// Toggle play / pause on the live player.
///
/// Resume is only valid when the audio engine actually holds a loaded stream.
/// When the player has NO loaded audio but the queue has a current track —
/// e.g. a freshly materialized QConnect renderer queue whose cursor sits on a
/// track that was never loaded, or a cold cursor after the queue ended — a
/// bare `resume()` fails with "cannot resume - no audio data available" and the
/// user sees a dead Play button. In that case LOAD and play the current queue
/// track instead, so Play works from a cold cursor. A normal pause leaves the
/// stream loaded (`has_loaded_audio` stays true), so the pause/resume path is
/// unchanged.
pub fn toggle_play_pause(runtime: Runtime, weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    handle.spawn(async move {
        if runtime.core().get_playback_state().is_playing {
            if let Err(e) = runtime.core().pause() {
                log::error!("[qbz-slint] playback: pause failed: {e}");
            } else {
                // Park the visualizer producer on the edge, ahead of the
                // 450ms poll tick that mirrors the flag onto the bar.
                set_viz_paused(&runtime, true);
            }
            // Persist the paused position so a restart resumes near where the
            // user stopped (no-op unless `persist_session` is on).
            crate::session_persist::capture_and_save(&runtime).await;
            return;
        }
        // Not playing: resume an existing stream, or cold-start the current
        // queue track when nothing is loaded.
        if runtime.core().player().has_loaded_audio() {
            if let Err(e) = runtime.core().resume() {
                log::error!("[qbz-slint] playback: resume failed: {e}");
            } else {
                // Wake the producer on the edge — resume must feel instant.
                set_viz_paused(&runtime, false);
            }
            return;
        }
        match runtime.core().current_track().await {
            Some(track) => {
                log::info!(
                    "[qbz-slint] playback: play with no loaded audio -> cold-starting current track {}",
                    track.id
                );
                after_track_change(&runtime, &weak, track.id).await;
                refresh_sidebar(true);
            }
            None => {
                log::info!("[qbz-slint] playback: toggle play ignored (no loaded audio, empty queue)");
            }
        }
    });
}

/// Advance to the next queue track and play it. Offline, unavailable
/// tracks are skipped (bounded — see `advance_to_playable`).
pub fn next(runtime: Runtime, weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    handle.spawn(async move {
        let Some(track) = advance_to_playable(&runtime, &weak, true).await else {
            log::info!("[qbz-slint] playback: end of queue");
            return;
        };
        let track_id = track.id;
        after_track_change(&runtime, &weak, track_id).await;
        refresh_sidebar(true);
    });
}

/// Go to the previous queue track and play it. Offline, unavailable
/// tracks are skipped (bounded — see `advance_to_playable`).
pub fn previous(runtime: Runtime, weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    handle.spawn(async move {
        let Some(track) = advance_to_playable(&runtime, &weak, false).await else {
            log::info!("[qbz-slint] playback: start of queue");
            return;
        };
        let track_id = track.id;
        after_track_change(&runtime, &weak, track_id).await;
        refresh_sidebar(true);
    });
}

/// Seek to `fraction` (0..1) of the current track's duration.
pub fn seek(runtime: Runtime, handle: tokio::runtime::Handle, fraction: f32) {
    handle.spawn(async move {
        let state = runtime.core().get_playback_state();
        if state.duration == 0 {
            return;
        }
        let fraction = fraction.clamp(0.0, 1.0);
        let position = (fraction as f64 * state.duration as f64).round() as u64;
        if let Err(e) = runtime.core().seek(position) {
            log::error!("[qbz-slint] playback: seek failed: {e}");
        }
    });
}
