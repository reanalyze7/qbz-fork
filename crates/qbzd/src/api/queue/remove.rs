use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{err_json, json, ApiState};

/// `index` for `remove` — distinct messages for a MISSING field vs a
/// present-but-wrong-type one (a string or negative `index` is not "you
/// forgot the field"; §1.4 error voice wants the message to name the actual
/// fault).
pub(super) fn parse_remove_index(body: &Value) -> Result<usize, (String, String)> {
    let hint = "body: {\"index\": 3}".to_string();
    match body.get("index") {
        None => Err(("remove requires an 'index' field".into(), hint)),
        Some(v) => match v.as_u64() {
            Some(i) => Ok(i as usize),
            None => Err(("'index' must be a non-negative integer".into(), hint)),
        },
    }
}

/// The pure remove-index decision (brief step 1: "the remove-playing-index
/// rejection body"). Bounds-check wins over the playing-index check when
/// both would fire (an out-of-range index cannot simultaneously BE the
/// playing index, so order does not matter in practice — bounds first reads
/// more naturally as "does this index exist at all").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RemoveCheck {
    Ok,
    OutOfRange,
    PlayingIndex,
}

pub(super) fn check_remove_index(index: usize, total_tracks: usize, current_index: Option<usize>) -> RemoveCheck {
    if index >= total_tracks {
        return RemoveCheck::OutOfRange;
    }
    if current_index == Some(index) {
        return RemoveCheck::PlayingIndex;
    }
    RemoveCheck::Ok
}

/// `POST /api/queue/remove` (02 §3.3.15). Body `{"index": N}`, 0-based —
/// SAME space as `current_index` (no CLI-boundary shift here). Errors:
/// 404 `not_found` (index out of range), 400 `bad_request` (index is the
/// playing track — the verbatim hint below is quoted §3.3.15; also a
/// missing or non-integer `index` field, with distinct messages).
pub fn remove(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    let index = match parse_remove_index(body) {
        Ok(i) => i,
        Err((message, hint)) => return err_json(400, "bad_request", &message, &hint),
    };

    let queue = state.rt.block_on(state.runtime.core().get_queue_state_full());
    match check_remove_index(index, queue.total_tracks, queue.current_index) {
        RemoveCheck::OutOfRange => {
            return err_json(
                404,
                "not_found",
                &format!("queue index {index} is out of range"),
                "check: qbzd queue list",
            )
        }
        RemoveCheck::PlayingIndex => {
            return err_json(
                400,
                "bad_request",
                &format!("index {index} is the playing track"),
                "use: qbzd next, or qbzd queue clear",
            )
        }
        RemoveCheck::Ok => {}
    }

    match state.rt.block_on(state.runtime.core().remove_track(index)) {
        Some(track) => {
            let total_tracks =
                state.rt.block_on(state.runtime.core().get_queue_state()).total_tracks;
            json(200, serde_json::json!({"removed": track.id, "total_tracks": total_tracks}))
        }
        // Narrow race, acknowledged: the bounds/playing check above and this
        // mutation are two separate core calls, and while the API serving
        // thread is single-threaded, the playback driver and QConnect tasks
        // mutate the queue independently — an auto-advance (or a remote
        // command) landing between the two calls can shrink the queue or
        // shift `current_index`, so this branch IS reachable, and a remove
        // can land on what just BECAME the playing row. Both degradations
        // are benign (a 404 here; a queue edit the §3.3.15 gate was one tick
        // too late to refuse) — never a panic, never corruption
        // (`QueueManager::remove_track` re-checks bounds under its own lock).
        // TODO(converge): atomic remove-guard primitive in qbz-core (P1).
        None => err_json(
            404,
            "not_found",
            &format!("queue index {index} is out of range"),
            "check: qbzd queue list",
        ),
    }
}
