use crate::api::response::error_body;
use crate::api::routes_table::{P0_ROUTES, P1_ROUTES};

#[test]
fn route_table_matches_spec_count() {
    // 02-cli-and-api.md §3.2 — P0 = exactly 17 routes, FINAL; grows ONLY
    // with a shipped client. T1-T6 landed 3; T7 +9 = 12; T8 +4 = 16; T11
    // (this task) +1 = 17.
    assert_eq!(P0_ROUTES.len(), 17);
    assert!(P0_ROUTES.contains(&("GET", "/api/ping")));
    assert!(P0_ROUTES.contains(&("GET", "/api/info")));
    assert!(P0_ROUTES.contains(&("GET", "/api/status")));
    assert!(P0_ROUTES.contains(&("GET", "/api/now-playing")));
    assert!(P0_ROUTES.contains(&("POST", "/api/playback/play")));
    assert!(P0_ROUTES.contains(&("POST", "/api/playback/pause")));
    assert!(P0_ROUTES.contains(&("POST", "/api/playback/toggle")));
    assert!(P0_ROUTES.contains(&("POST", "/api/playback/stop")));
    assert!(P0_ROUTES.contains(&("POST", "/api/playback/next")));
    assert!(P0_ROUTES.contains(&("POST", "/api/playback/previous")));
    assert!(P0_ROUTES.contains(&("POST", "/api/playback/seek")));
    assert!(P0_ROUTES.contains(&("POST", "/api/playback/volume")));
    assert!(P0_ROUTES.contains(&("GET", "/api/queue")));
    assert!(P0_ROUTES.contains(&("POST", "/api/queue/add")));
    assert!(P0_ROUTES.contains(&("POST", "/api/queue/remove")));
    assert!(P0_ROUTES.contains(&("POST", "/api/queue/clear")));
    assert!(P0_ROUTES.contains(&("POST", "/api/settings/reload")));
}

#[test]
fn p1_route_table_grows_only_with_a_shipped_caller() {
    // 02-cli-and-api.md §3.4 — each P1 route lands with its CLI verb (the
    // §3.1.4 HARD RULE, applied to the content-verb door). Row 19:
    // GET /api/search — caller: `qbzd search`. Count is pinned so a route
    // with no caller cannot creep in; P1 must never overlap P0.
    assert_eq!(P1_ROUTES.len(), 25);
    assert!(P1_ROUTES.contains(&("GET", "/api/events"))); // caller: `qbzd watch`
    assert!(P1_ROUTES.contains(&("GET", "/api/artwork/current"))); // caller: `qbzd art`
    assert!(P1_ROUTES.contains(&("GET", "/api/discover")));
    assert!(P1_ROUTES.contains(&("POST", "/api/reco/playlist")));
    assert!(P1_ROUTES.contains(&("GET", "/api/favorites")));
    assert!(P1_ROUTES.contains(&("POST", "/api/favorites/add")));
    assert!(P1_ROUTES.contains(&("POST", "/api/favorites/remove")));
    assert!(P1_ROUTES.contains(&("GET", "/api/playlists")));
    assert!(P1_ROUTES.contains(&("GET", "/api/playlist")));
    assert!(P1_ROUTES.contains(&("POST", "/api/playlist/create")));
    assert!(P1_ROUTES.contains(&("POST", "/api/playlist/update")));
    assert!(P1_ROUTES.contains(&("POST", "/api/playlist/delete")));
    assert!(P1_ROUTES.contains(&("POST", "/api/playlist/tracks/add")));
    assert!(P1_ROUTES.contains(&("POST", "/api/playlist/tracks/remove")));
    assert!(P1_ROUTES.contains(&("GET", "/api/search")));
    assert!(P1_ROUTES.contains(&("POST", "/api/play")));
    assert!(P1_ROUTES.contains(&("GET", "/api/album")));
    assert!(P1_ROUTES.contains(&("GET", "/api/artist")));
    assert!(P1_ROUTES.contains(&("GET", "/api/similar")));
    assert!(P1_ROUTES.contains(&("GET", "/api/suggest")));
    assert!(P1_ROUTES.contains(&("POST", "/api/playback/shuffle")));
    assert!(P1_ROUTES.contains(&("POST", "/api/playback/repeat")));
    assert!(P1_ROUTES.contains(&("POST", "/api/queue/move")));
    assert!(P1_ROUTES.contains(&("POST", "/api/queue/jump")));
    assert!(P1_ROUTES.contains(&("POST", "/api/queue/stop-after")));
    for r in P1_ROUTES {
        assert!(!P0_ROUTES.contains(r), "{r:?} is duplicated across P0 and P1");
    }
}

#[test]
fn error_envelope_is_the_nested_shape_with_code() {
    // 02 §3.1.3 — the on-the-wire shape the CLI reads via `error.code`.
    let body = error_body("origin_forbidden", "refused", "not a browser API");
    assert_eq!(body["error"]["code"], "origin_forbidden");
    assert_eq!(body["error"]["message"], "refused");
    assert_eq!(body["error"]["hint"], "not a browser API");
}
