use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::api::{err_json, json, ApiState};

/// `POST /api/playback/shuffle` (CONSOLE). Body `{"mode": "on"|"off"|"toggle"}`
/// (default `toggle`). Returns the resulting shuffle state. No auth — a
/// queue-mode toggle touches no Qobuz session; state also surfaces in
/// `/api/status` and `/api/now-playing`.
pub fn shuffle(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    let mode = body.get("mode").and_then(|v| v.as_str()).unwrap_or("toggle");
    let enabled = match mode {
        "on" => {
            state.rt.block_on(state.runtime.core().set_shuffle(true));
            true
        }
        "off" => {
            state.rt.block_on(state.runtime.core().set_shuffle(false));
            false
        }
        "toggle" => state.rt.block_on(state.runtime.core().toggle_shuffle()),
        other => {
            return err_json(400, "bad_request", &format!("invalid mode '{other}'"), "mode: on | off | toggle")
        }
    };
    json(200, serde_json::json!({"shuffle": enabled}))
}

/// `POST /api/playback/repeat` (CONSOLE). Body `{"mode": "off"|"all"|"one"}`.
/// No auth. Returns the applied mode.
pub fn repeat(state: &ApiState, body: &Value) -> Response<Cursor<Vec<u8>>> {
    let mode = body.get("mode").and_then(|v| v.as_str()).unwrap_or("");
    let rm = match mode {
        "off" => qbz_models::RepeatMode::Off,
        "all" => qbz_models::RepeatMode::All,
        "one" => qbz_models::RepeatMode::One,
        other => {
            return err_json(400, "bad_request", &format!("invalid repeat mode '{other}'"), "mode: off | all | one")
        }
    };
    state.rt.block_on(state.runtime.core().set_repeat_mode(rm));
    json(200, serde_json::json!({"repeat": mode}))
}

pub(super) fn repeat_str(mode: qbz_models::RepeatMode) -> String {
    match mode {
        qbz_models::RepeatMode::Off => "off",
        qbz_models::RepeatMode::All => "all",
        qbz_models::RepeatMode::One => "one",
    }
    .to_string()
}
