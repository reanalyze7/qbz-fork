use crate::api::gate::{access_gate, constant_time_eq, GateReject};
use crate::api::response::canon_volume;
use crate::api::routes_table::P0_ROUTES;

fn code(r: Option<GateReject>) -> Option<&'static str> {
    r.map(|r| match r {
        GateReject::OriginForbidden => "origin_forbidden",
        GateReject::InvalidToken => "invalid_token",
    })
}

#[test]
fn constant_time_eq_matches_only_identical_slices() {
    assert!(constant_time_eq(b"Bearer s3cret", b"Bearer s3cret"));
    assert!(!constant_time_eq(b"Bearer s3cret", b"Bearer wrong"));
    assert!(!constant_time_eq(b"Bearer s3cret", b"Bearer s3cret-extra"));
    assert!(!constant_time_eq(b"", b"x"));
}

#[test]
fn origin_header_is_refused_on_every_route_including_ping() {
    // Step 4(a): an Origin header → 403 origin_forbidden everywhere, even
    // /api/ping, and even in the open (token=None) default.
    for (m, p) in P0_ROUTES {
        assert_eq!(
            code(access_gate(true, m, p, None, None)),
            Some("origin_forbidden"),
            "{m} {p} with Origin must be refused"
        );
    }
    // ...and the Origin shield wins even when a valid Bearer is present.
    assert_eq!(
        code(access_gate(true, "GET", "/api/ping", Some("Bearer s3cret"), Some("s3cret"))),
        Some("origin_forbidden")
    );
}

#[test]
fn open_mode_answers_every_route_without_auth() {
    // Step 4(b): token=None → no auth machinery; nothing is rejected.
    for (m, p) in P0_ROUTES {
        assert!(access_gate(false, m, p, None, None).is_none(), "{m} {p} must be open");
    }
}

#[test]
fn opt_in_token_rejects_missing_or_wrong_bearer_but_never_ping() {
    // Step 4(c): token=Some → missing/wrong bearer is 401 on non-ping routes;
    // /api/ping stays 200 (exempt); the correct bearer passes.
    let tok = Some("s3cret");
    assert_eq!(code(access_gate(false, "GET", "/api/status", None, tok)), Some("invalid_token"));
    assert_eq!(
        code(access_gate(false, "GET", "/api/status", Some("Bearer nope"), tok)),
        Some("invalid_token")
    );
    assert!(access_gate(false, "GET", "/api/status", Some("Bearer s3cret"), tok).is_none());
    // /api/ping answers even with no/ wrong bearer.
    assert!(access_gate(false, "GET", "/api/ping", None, tok).is_none());
    assert!(access_gate(false, "GET", "/api/ping", Some("Bearer nope"), tok).is_none());
}

#[test]
fn canon_volume_pins_0_8_exactly_no_f32_widening() {
    // `serde_json::Number::from_f32` widens f32→f64 (`0.8f32` would
    // serialize raw as `0.800000011920929`); `canon_volume` must not.
    assert_eq!(serde_json::to_string(&canon_volume(0.8f32)).unwrap(), "0.8");
    assert_eq!(serde_json::to_string(&canon_volume(1.0f32)).unwrap(), "1.0");
    assert_eq!(serde_json::to_string(&canon_volume(0.0f32)).unwrap(), "0.0");
    assert_eq!(serde_json::to_string(&canon_volume(0.75f32)).unwrap(), "0.75");
}
