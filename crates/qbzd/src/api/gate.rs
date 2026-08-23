use std::io::Cursor;

use tiny_http::Response;

use super::response::err_json;

/// A pre-routing rejection. Carried as a small enum so [`access_gate`] stays a
/// pure decision (unit-testable without a tiny_http `Request`, which has no
/// public constructor) while `route` renders the normative envelope.
pub(super) enum GateReject {
    OriginForbidden,
    InvalidToken,
}

impl GateReject {
    pub(super) fn response(&self) -> Response<Cursor<Vec<u8>>> {
        match self {
            GateReject::OriginForbidden => err_json(
                403,
                "origin_forbidden",
                "requests with an Origin header are refused",
                "the control plane is not a browser API",
            ),
            GateReject::InvalidToken => err_json(
                401,
                "invalid_token",
                "missing or wrong bearer token",
                "set QBZD_TOKEN or check [server] token in qbzd.toml",
            ),
        }
    }
}

/// The pre-routing access decision (02 §3.1.2): Origin shield always on; the
/// opt-in Bearer required on every route except `GET /api/ping` when `token`
/// is `Some`. `None` = open (no auth machinery). Returns `Some(_)` to reject.
pub(super) fn access_gate(
    has_origin: bool,
    method: &str,
    path: &str,
    auth_header: Option<&str>,
    token: Option<&str>,
) -> Option<GateReject> {
    if has_origin {
        return Some(GateReject::OriginForbidden);
    }
    if let Some(secret) = token {
        let is_ping = method == "GET" && path == "/api/ping";
        let expected = format!("Bearer {secret}");
        let ok = auth_header
            .map(|v| constant_time_eq(v.as_bytes(), expected.as_bytes()))
            .unwrap_or(false);
        if !is_ping && !ok {
            return Some(GateReject::InvalidToken);
        }
    }
    None
}

pub(super) fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
