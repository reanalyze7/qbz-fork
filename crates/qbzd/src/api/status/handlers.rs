use std::io::Cursor;

use tiny_http::Response;

use super::assemble::assemble_live;

// ============================ live handlers (T6) ============================

/// `GET /api/info` (02 §3.3.2) — identity for the CLI's version-skew diagnosis
/// (§1.6, the single sanctioned second request). Deliberately minimal.
pub fn info(state: &crate::api::ApiState) -> Response<Cursor<Vec<u8>>> {
    let uptime = state
        .shared
        .lock()
        .ok()
        .map(|s| s.started_at.elapsed().as_secs())
        .unwrap_or(0);
    crate::api::json(
        200,
        serde_json::json!({
            "app": "qbzd",
            "version": env!("CARGO_PKG_VERSION"),
            "api_version": crate::API_VERSION,
            "bind": state.bind,
            "uptime_secs": uptime,
            "data_root": state.roots.data.display().to_string(),
        }),
    )
}

/// `GET /api/status` (02 §3.3.3) — the composite daemon status. ALWAYS 200;
/// the CLI maps degradation (needs_auth, missing device) to exit codes.
pub fn status(state: &crate::api::ApiState) -> Response<Cursor<Vec<u8>>> {
    let doc = assemble_live(state);
    let mut value = serde_json::to_value(&doc).unwrap_or_else(|_| serde_json::json!({}));
    // `StatusDoc.playback.volume` is f32; plain `to_value` widens it via
    // `Number::from_f32` (f32→f64, `0.8` → `0.800000011920929`). Overwrite
    // with the canonical form — see `super::canon_volume`.
    if let Some(vol) = value.pointer_mut("/playback/volume") {
        *vol = crate::api::canon_volume(doc.playback.volume);
    }
    crate::api::json(200, value)
}
