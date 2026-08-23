use std::io::Cursor;

use tiny_http::{Request, Response};

use super::gate::access_gate;
use super::response::{err_json, json};
use super::router_lists::route_lists;
use super::state::ApiState;
use super::{artwork, browse, discover, fav, play, playback, reco, search, settings, status};

pub(super) fn route(state: &ApiState, req: &mut Request) -> Response<Cursor<Vec<u8>>> {
    let method = req.method().as_str().to_owned();
    let url = req.url().to_owned();
    let mut url_parts = url.splitn(2, '?');
    let path = url_parts.next().unwrap_or("").to_owned();
    let query = url_parts.next().unwrap_or("").to_owned();
    let has_origin = req.headers().iter().any(|h| h.field.equiv("Origin"));
    let auth_header = req
        .headers()
        .iter()
        .find(|h| h.field.equiv("Authorization"))
        .map(|h| h.value.as_str());

    // Origin shield (ALWAYS on) + opt-in [server] token — one pre-routing gate,
    // /api/ping exempt from the token only (§3.1.2).
    if let Some(reject) = access_gate(has_origin, &method, &path, auth_header, state.token.as_deref())
    {
        return reject.response();
    }

    match (method.as_str(), path.as_str()) {
        ("GET", "/api/ping") => json(
            200,
            serde_json::json!({"ok": true, "app": "qbzd", "api_version": crate::API_VERSION}),
        ),
        ("GET", "/api/info") => status::info(state),
        ("GET", "/api/status") => status::status(state),
        ("GET", "/api/now-playing") => playback::now_playing(state),
        ("POST", "/api/playback/play") => playback::play(state),
        ("POST", "/api/playback/pause") => playback::pause(state),
        ("POST", "/api/playback/toggle") => playback::toggle(state),
        ("POST", "/api/playback/stop") => playback::stop(state),
        ("POST", "/api/playback/next") => playback::next(state),
        ("POST", "/api/playback/previous") => playback::previous(state),
        ("POST", "/api/playback/seek") => {
            let body = read_json_body(req);
            playback::seek(state, &body)
        }
        ("POST", "/api/playback/volume") => {
            let body = read_json_body(req);
            playback::volume(state, &body)
        }
        ("POST", "/api/playback/shuffle") => {
            let body = read_json_body(req);
            playback::shuffle(state, &body)
        }
        ("POST", "/api/playback/repeat") => {
            let body = read_json_body(req);
            playback::repeat(state, &body)
        }
        ("GET", "/api/search") => search::search(state, &query),
        ("POST", "/api/play") => {
            let body = read_json_body(req);
            play::play(state, &body)
        }
        ("GET", "/api/album") => browse::album(state, &query),
        ("GET", "/api/artist") => browse::artist(state, &query),
        ("GET", "/api/similar") => browse::similar(state, &query),
        ("GET", "/api/suggest") => browse::suggest(state, &query),
        ("GET", "/api/discover") => discover::discover(state, &query),
        ("GET", "/api/artwork/current") => artwork::current(state),
        ("POST", "/api/reco/playlist") => {
            let body = read_json_body(req);
            reco::playlist(state, &body)
        }
        ("GET", "/api/favorites") => fav::list(state, &query),
        ("POST", "/api/favorites/add") => {
            let body = read_json_body(req);
            fav::add(state, &body)
        }
        ("POST", "/api/favorites/remove") => {
            let body = read_json_body(req);
            fav::remove(state, &body)
        }
        ("POST", "/api/settings/reload") => settings::reload(state),
        (method, path) => match route_lists(state, method, path, &query, req) {
            Some(resp) => resp,
            None => err_json(404, "not_found", "unknown route", "see qbzd --help"),
        },
    }
}

/// Read and parse a request body as JSON (T7's seek/volume POST bodies).
/// An unreadable or absent body parses to `Value::Null` — the route handlers
/// treat a missing expected field as `400 bad_request`, never a panic.
pub(super) fn read_json_body(req: &mut Request) -> serde_json::Value {
    let mut buf = String::new();
    let _ = req.as_reader().read_to_string(&mut buf);
    serde_json::from_str(&buf).unwrap_or(serde_json::Value::Null)
}
