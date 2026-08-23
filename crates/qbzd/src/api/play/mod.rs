// crates/qbzd/src/api/play/ — POST /api/play (02-cli-and-api.md §2.3/§3.4
// row 23, P1). The headline "originate playback" verb: qbzd resolves a piece
// of content — a track, album, playlist, artist, or a Qobuz URL — materializes
// it into the queue server-side, and starts audio through the SHIPPED driver
// ritual. This is what turns the daemon from a receiver into a source.
//
// Materialization is server-side and never trusts a client-built QueueTrack
// (same discipline as queue::add): each Track comes from the core
// (get_album / get_playlist / get_artist_tracks / get_tracks_batch) and is
// mapped by the shared queue::track_to_queue_track. Audio ALWAYS starts through
// core.play_track_resolved + save_session_now — the exact cold-start tail
// playback::cold_start uses — never a bare cursor move (control-surface §2.2);
// the protected qbz-player/qbz-audio crates are untouched.
//
// Body: one of {"track_id": N} | {"album_id": "..."} | {"playlist_id": N} |
// {"artist_id": N} | {"url": "https://open.qobuz.com/..."}, plus optional
// {"index": N} (0-based start position within the resolved list). A URL wins
// over the id fields and is resolved to one of the id kinds first. Errors:
// 409 needs_auth, 400 bad_request (no selector / bad URL), 404 not_found
// (unknown id / empty resolution), 503 audio_unavailable (start failed).
mod selector;
#[cfg(test)]
mod tests;
mod util;

use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use qbz_models::{QueueTrack, Track};

use super::queue::track_to_queue_track;
use super::{err_json, json, ApiState};

use selector::{fetch_tracks, parse_selector};
use util::{auth_gate, clamp_index, summary};

pub fn play(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }

    let selector = match parse_selector(body) {
        Ok(s) => s,
        Err((message, hint)) => return err_json(400, "bad_request", &message, &hint),
    };

    let (tracks, context) = match fetch_tracks(state, &selector) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    start_resolved(state, tracks, context, body.get("index").and_then(|v| v.as_u64()))
}

/// Materialize resolved catalog `tracks` into the queue and cold-start the
/// chosen one through the SHIPPED ritual (set_queue → play_track_resolved →
/// save_session_now) — never a bare cursor move (control-surface §2.2). The
/// protected qbz-player/qbz-audio crates are untouched. `context` stamps
/// "playing from" provenance (context_kind ∈ album|artist|playlist —
/// qbz-models/src/playback.rs:60-71); `start_index` is the 0-based start
/// position.
pub(crate) fn start_resolved(
    state: &ApiState,
    tracks: Vec<Track>,
    context: Option<(&'static str, String)>,
    start_index: Option<u64>,
) -> Response<Cursor<Vec<u8>>> {
    if tracks.is_empty() {
        return err_json(404, "not_found", "nothing to play", "check the id: qbzd search <QUERY>");
    }

    let mut queue_tracks: Vec<QueueTrack> = tracks.iter().map(track_to_queue_track).collect();
    if let Some((kind, id)) = &context {
        for qt in &mut queue_tracks {
            qt.context_kind = Some((*kind).to_string());
            qt.context_id = Some(id.clone());
        }
    }

    let total = queue_tracks.len();
    let start = clamp_index(start_index, total);
    let start_track_id = queue_tracks[start].id;
    let start_summary = summary(&queue_tracks[start]);

    state
        .rt
        .block_on(state.runtime.core().set_queue(queue_tracks, Some(start)));

    let quality = super::playback::resolve_quality(state);
    // Spawn-and-ack (see `playback::advance`): set_queue is cheap and stays
    // synchronous; the resolve+play leg runs on the tokio runtime so a slow
    // fetch can't starve the single-threaded API. A load failure latches into
    // `last_errors.stream` (visible via `qbzd status`) instead of a 503.
    let runtime = std::sync::Arc::clone(&state.runtime);
    let shared = std::sync::Arc::clone(&state.shared);
    state.rt.spawn(async move {
        let played = runtime
            .core()
            .play_track_resolved(start_track_id, quality, None, None, 0)
            .await;
        if let Err(err) = played {
            log::error!("[api] play start of {start_track_id} failed: {err}");
            if let Ok(mut s) = shared.lock() {
                s.last_errors.stream = Some(format!("play: {err}"));
            }
            return;
        }
        qbz_app::playback_driver::save_session_now(runtime.as_ref()).await;
    });

    json(
        200,
        serde_json::json!({
            "queued": total,
            "started": true,
            "index": start,
            "track": start_summary,
        }),
    )
}
