use std::io::Cursor;

use tiny_http::{Request, Response};

use super::router::read_json_body;
use super::state::ApiState;
use super::{playlist, queue};

/// The playlist/queue route arms, split out of [`super::router::route`] purely
/// to keep that file under the line budget — same dispatch, no behavior change.
pub(super) fn route_lists(
    state: &ApiState,
    method: &str,
    path: &str,
    query: &str,
    req: &mut Request,
) -> Option<Response<Cursor<Vec<u8>>>> {
    Some(match (method, path) {
        ("GET", "/api/playlists") => playlist::list(state),
        ("GET", "/api/playlist") => playlist::show(state, query),
        ("POST", "/api/playlist/create") => {
            let body = read_json_body(req);
            playlist::create(state, &body)
        }
        ("POST", "/api/playlist/update") => {
            let body = read_json_body(req);
            playlist::update(state, &body)
        }
        ("POST", "/api/playlist/delete") => {
            let body = read_json_body(req);
            playlist::delete(state, &body)
        }
        ("POST", "/api/playlist/tracks/add") => {
            let body = read_json_body(req);
            playlist::tracks_add(state, &body)
        }
        ("POST", "/api/playlist/tracks/remove") => {
            let body = read_json_body(req);
            playlist::tracks_remove(state, &body)
        }
        ("GET", "/api/queue") => queue::list(state, query),
        ("POST", "/api/queue/add") => {
            let body = read_json_body(req);
            queue::add(state, &body)
        }
        ("POST", "/api/queue/remove") => {
            let body = read_json_body(req);
            queue::remove(state, &body)
        }
        ("POST", "/api/queue/clear") => {
            let body = read_json_body(req);
            queue::clear(state, &body)
        }
        ("POST", "/api/queue/move") => {
            let body = read_json_body(req);
            queue::reorder(state, &body)
        }
        ("POST", "/api/queue/jump") => {
            let body = read_json_body(req);
            queue::jump(state, &body)
        }
        ("POST", "/api/queue/stop-after") => {
            let body = read_json_body(req);
            queue::stop_after(state, &body)
        }
        _ => return None,
    })
}
