use std::io::Cursor;
use std::sync::Arc;

use tiny_http::Response;

use crate::api::{err_json, ApiState};

use super::errors::auth_gate;
use super::resolve_quality;

/// `play`/`toggle`'s cold-start branch: gate on NeedsAuth, resolve the
/// current queue track, then SPAWN resolve+play+persist — the same ritual
/// tail `advance_and_play` runs, minus the cursor-move (we're playing the
/// CURRENT track, not advancing to a new one) and the gapless prefetch (the
/// running driver's tick-based `ArmGapless` picks that up on a later tick
/// once playback is underway).
///
/// Spawn-and-ack (see `advance`): the gates (auth, empty queue) stay
/// synchronous so their documented errors are immediate; the network-bound
/// load leg runs on the tokio runtime and latches failures into
/// `last_errors.stream` instead of a 5xx. Ok(()) means "load queued" — the
/// callers answer `{"state": "loading"}`.
pub(super) fn cold_start(state: &ApiState) -> Result<(), Response<Cursor<Vec<u8>>>> {
    if let Some(resp) = auth_gate(state) {
        return Err(resp);
    }
    let queue = state.rt.block_on(state.runtime.core().get_queue_state());
    let Some(track) = queue.current_track else {
        // No documented error code fits "empty queue" exactly; audio_unavailable
        // (503, exit 5) is the closest frozen taxonomy match — "can't produce
        // audio because there is nothing queued" — and the hint names the fix.
        return Err(err_json(
            503,
            "audio_unavailable",
            "queue is empty, nothing to play",
            "queue a track first: qbzd queue add <TRACK_ID>",
        ));
    };
    let track_id = track.id;
    let quality = resolve_quality(state);
    let runtime = std::sync::Arc::clone(&state.runtime);
    let shared = Arc::clone(&state.shared);
    state.rt.spawn(async move {
        let played = runtime
            .core()
            .play_track_resolved(track_id, quality, None, None, 0)
            .await;
        if let Err(err) = played {
            log::error!("[api] cold-start play of {track_id} failed: {err}");
            if let Ok(mut s) = shared.lock() {
                s.last_errors.stream = Some(format!("play: {err}"));
            }
            return;
        }
        qbz_app::playback_driver::save_session_now(runtime.as_ref()).await;
    });
    Ok(())
}
