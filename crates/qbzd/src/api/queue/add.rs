use std::io::Cursor;

use qbz_models::QueueTrack;
use serde_json::Value;
use tiny_http::Response;

use crate::api::{err_json, json, ApiState};

use super::mapping::track_to_queue_track;
use super::shared::auth_gate;

/// `position` for `add` — strict literal match (§3.3.14: `"end"|"next"`,
/// default `"end"` when the field is absent). Anything else — an unknown
/// literal, or a non-string — is a 400, never a silent fall-through to
/// "end" (a typo'd `"nxet"` appending to the queue tail instead of playing
/// next would read as broken).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AddPosition {
    End,
    Next,
}

/// Strict `track_ids` parse — ANY non-u64 element is a 400 naming the
/// offending (0-based JSON array) position. The previous `filter_map` parse
/// silently dropped malformed elements: "add 3, enqueue 2" is data loss the
/// caller never sees. Runs BEFORE any core call, so a rejected body leaves
/// the queue untouched.
pub(super) fn parse_track_ids(body: &Value) -> Result<Vec<u64>, (String, String)> {
    let hint = "body: {\"track_ids\": [176544872]}".to_string();
    let arr = match body.get("track_ids").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return Err(("add requires a 'track_ids' array".into(), hint)),
    };
    if arr.is_empty() {
        return Err(("'track_ids' must not be empty".into(), hint));
    }
    let mut ids = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        match v.as_u64() {
            Some(id) => ids.push(id),
            None => {
                return Err((
                    format!("track_ids[{i}] is not an unsigned integer track id"),
                    hint,
                ))
            }
        }
    }
    Ok(ids)
}

pub(super) fn parse_position(body: &Value) -> Result<AddPosition, (String, String)> {
    let hint = "body: {\"track_ids\": [176544872], \"position\": \"next\"}".to_string();
    match body.get("position") {
        None => Ok(AddPosition::End),
        Some(v) => match v.as_str() {
            Some("end") => Ok(AddPosition::End),
            Some("next") => Ok(AddPosition::Next),
            _ => Err((format!("invalid position {v} — use \"end\" or \"next\""), hint)),
        },
    }
}

/// `POST /api/queue/add` (02 §3.3.14). Body `{"track_ids": [...], "position":
/// "end"|"next"}` (`position` default `"end"`). Errors: 409 `needs_auth`,
/// 404 `not_found` (any unresolvable id — resolution stops at the first
/// failure, nothing partially added), 400 `bad_request` (malformed
/// `track_ids` element or unknown `position` literal — both parses are
/// STRICT and run before any core call, so a rejected body never partially
/// mutates the queue).
pub fn add(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }

    let track_ids = match parse_track_ids(body) {
        Ok(ids) => ids,
        Err((message, hint)) => return err_json(400, "bad_request", &message, &hint),
    };
    let position = match parse_position(body) {
        Ok(p) => p,
        Err((message, hint)) => return err_json(400, "bad_request", &message, &hint),
    };

    let mut resolved: Vec<QueueTrack> = Vec::with_capacity(track_ids.len());
    for id in &track_ids {
        match state.rt.block_on(state.runtime.core().get_track(*id)) {
            Ok(track) => resolved.push(track_to_queue_track(&track)),
            Err(_) => {
                return err_json(
                    404,
                    "not_found",
                    &format!("track {id} not found"),
                    "check the track id: qbzd search <QUERY>",
                )
            }
        }
    }

    let added = resolved.len();
    let tracks_json: Vec<Value> = resolved
        .iter()
        .map(|t| serde_json::to_value(t).unwrap_or(Value::Null))
        .collect();

    if position == AddPosition::Next {
        // `add_track_next` always inserts immediately after the current
        // track, so reverse iteration is what lands multiple tracks in
        // request order (matches the desktop's multi-add-next convention).
        for track in resolved.into_iter().rev() {
            state.rt.block_on(state.runtime.core().add_track_next(track));
        }
    } else {
        state.rt.block_on(state.runtime.core().add_tracks(resolved));
    }

    let total_tracks = state.rt.block_on(state.runtime.core().get_queue_state()).total_tracks;
    json(
        200,
        serde_json::json!({"added": added, "total_tracks": total_tracks, "tracks": tracks_json}),
    )
}
