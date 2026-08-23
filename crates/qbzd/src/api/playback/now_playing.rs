use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{canon_volume, json, ApiState};

use super::errors::auth_gate;
use super::queue_modes::repeat_str;
use super::volume::nominal_volume;

/// `GET /api/now-playing` (02 §3.3.4). `playback` is the serialized
/// `PlaybackEvent` (qbz-player/src/player/mod.rs:925) with `shuffle`/`repeat`
/// filled in from the queue (the player itself leaves them `None` — "Set by
/// caller with access to queue state") plus the daemon-owned `muted` field,
/// plus an ADDITIVE `queue_len` (02 §3.1.4 allows additive fields within
/// api_version 1; needed so `qbzd now`'s stopped-state render, "stopped ·
/// queue 14 tracks", has a count — the documented playing-state example has
/// no queue count because nothing needs one while a track is loaded).
/// `track` is the current `QueueTrack`, or `null` when nothing is loaded.
pub fn now_playing(state: &ApiState) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }

    let player = state.runtime.core().player();
    let mut ev = player.get_playback_event();
    let queue = state.rt.block_on(state.runtime.core().get_queue_state());

    ev.shuffle = Some(queue.shuffle);
    ev.repeat = Some(repeat_str(queue.repeat));

    let (muted, nominal_volume) = nominal_volume(state, ev.volume);
    ev.volume = nominal_volume;

    let mut playback = serde_json::to_value(&ev).unwrap_or_else(|_| serde_json::json!({}));
    if let Value::Object(map) = &mut playback {
        map.insert("muted".into(), serde_json::json!(muted));
        map.insert("queue_len".into(), serde_json::json!(queue.total_tracks));
        // Overwrite the f32→f64-widened `volume` serde_json::to_value produced
        // above with the canonical (3-decimal) form — see `canon_volume`.
        map.insert("volume".into(), canon_volume(nominal_volume));
    }

    let track = queue
        .current_track
        .as_ref()
        .map(|t| serde_json::to_value(t).unwrap_or(Value::Null))
        .unwrap_or(Value::Null);

    json(200, serde_json::json!({"playback": playback, "track": track}))
}
