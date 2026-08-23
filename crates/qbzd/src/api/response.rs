use std::io::Cursor;

use tiny_http::{Header, Response};

/// Canonical JSON number for a volume level (0.0-1.0). `serde_json::Value`
/// backs f32 via `Number::from_f32`, which stores the value as `f as f64` —
/// the f32→f64 widening turns `0.8f32` into `0.800000011920929` on the wire.
/// 02-cli-and-api.md §2.2/§3.3.4 document plain `0.8`, and `--json` is the
/// frozen machine contract ("scripts parse this"), so every volume-bearing
/// response routes through this instead of a bare `json!(v)`. 3 decimals is
/// plenty of precision for a 0.0-1.0 level.
pub(crate) fn canon_volume(v: f32) -> serde_json::Value {
    let rounded = (v as f64 * 1000.0).round() / 1000.0;
    serde_json::json!(rounded)
}

/// A 2xx JSON response. `pub(crate)` so the per-route handlers in `status.rs`
/// share the exact same envelope framing.
pub(crate) fn json(status: u16, body: serde_json::Value) -> Response<Cursor<Vec<u8>>> {
    let bytes = serde_json::to_vec(&body).unwrap_or_else(|_| b"{}".to_vec());
    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
        .expect("static content-type header");
    Response::from_data(bytes)
        .with_status_code(status)
        .with_header(header)
}

/// The normative error envelope (02 §3.1.3): `{"error":{"code","message","hint"}}`.
/// The CLI keys its exit code off `code` (never raw HTTP status), and every hint
/// names the fix (§1.4 error voice). The G0 addendum's shorthand
/// `{"error":"origin_forbidden"}` is this same nested envelope — the uniform
/// §3.1.3 shape the CLI's `error_from_envelope` reads via `error.code`.
pub(crate) fn err_json(
    status: u16,
    code: &str,
    message: &str,
    hint: &str,
) -> Response<Cursor<Vec<u8>>> {
    json(status, error_body(code, message, hint))
}

pub(crate) fn error_body(code: &str, message: &str, hint: &str) -> serde_json::Value {
    serde_json::json!({"error": {"code": code, "message": message, "hint": hint}})
}
