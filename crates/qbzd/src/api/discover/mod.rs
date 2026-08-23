// crates/qbzd/src/api/discover/ — GET /api/discover (02 §2.3 CONSOLE). The
// editorial Discover rails, so a third-party client can replicate the QBZ
// Discover page WITHOUT the personalized recommendation engine (which needs
// per-user artist-vector / play-count stores the daemon deliberately does not
// open — qbz-app/src/settings/reco_store.rs, never installed here).
//
// One route, a `section` selector over the Qobuz-backed discover family:
//   index                -> get_discover_index (the composed home index)
//   most-streamed         -> /discover/mostStreamed   (Qobuz "most played")
//   new-releases          -> /discover/newReleases
//   press-awards          -> /discover/pressAward
//   qobuzissims           -> /discover/qobuzissims
//   album-of-the-week     -> /discover/albumOfTheWeek
//   ideal-discography     -> /discover/idealDiscography
//   playlists [?tag=]     -> get_discover_playlists
//   tags                  -> get_playlist_tags
//   release-watch [?release_type=artists|labels|awards] -> get_release_watch
//   featured  ?type=<t>   -> get_featured_albums (raw Qobuz featured_type)
//
// All auth-gated; all return the core's typed serde shapes verbatim (a stable
// --json contract); optional ?genre=ID,ID scopes the section by genre.
mod query;

use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use crate::state::AuthState;

use super::{err_json, json, ApiState};
use query::{get, limit_offset, pairs, parse_genre};

pub fn discover(state: &ApiState, query: &str) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }
    let p = pairs(query);
    let section = get(&p, "section").unwrap_or("index").to_string();
    let genre = parse_genre(get(&p, "genre"));
    let (limit, offset) = limit_offset(&p);
    let core = state.runtime.core();

    let data: Result<Value, Response<Cursor<Vec<u8>>>> = match section.as_str() {
        "index" => serialize(state.rt.block_on(core.get_discover_index(genre))),
        "most-streamed" => serialize(state.rt.block_on(core.get_discover_albums("/discover/mostStreamed", genre, offset, limit))),
        "new-releases" => serialize(state.rt.block_on(core.get_discover_albums("/discover/newReleases", genre, offset, limit))),
        "press-awards" => serialize(state.rt.block_on(core.get_discover_albums("/discover/pressAward", genre, offset, limit))),
        "qobuzissims" => serialize(state.rt.block_on(core.get_discover_albums("/discover/qobuzissims", genre, offset, limit))),
        "album-of-the-week" => serialize(state.rt.block_on(core.get_discover_albums("/discover/albumOfTheWeek", genre, offset, limit))),
        "ideal-discography" => serialize(state.rt.block_on(core.get_discover_albums("/discover/idealDiscography", genre, offset, limit))),
        "playlists" => {
            let tag = get(&p, "tag").map(|s| s.to_string());
            serialize(state.rt.block_on(core.get_discover_playlists(tag, genre, Some(limit), Some(offset))))
        }
        "tags" => serialize(state.rt.block_on(core.get_playlist_tags())),
        "release-watch" => {
            let rt = get(&p, "release_type").unwrap_or("artists");
            serialize(state.rt.block_on(core.get_release_watch(rt, limit, offset)))
        }
        "featured" => match get(&p, "type") {
            Some(ft) => {
                let g1 = genre.as_ref().and_then(|v| v.first().copied());
                serialize(state.rt.block_on(core.get_featured_albums(ft, limit, offset, g1)))
            }
            None => return err_json(400, "bad_request", "featured requires ?type=", "or use a named section, e.g. section=most-streamed"),
        },
        other => {
            return err_json(
                400,
                "bad_request",
                &format!("unknown discover section '{other}'"),
                "section: index | most-streamed | new-releases | press-awards | qobuzissims | album-of-the-week | ideal-discography | playlists | tags | release-watch",
            )
        }
    };

    match data {
        Ok(v) => json(200, serde_json::json!({"section": section, "data": v})),
        Err(resp) => resp,
    }
}

/// Serialize an already-awaited core result, or map an upstream error to 502.
fn serialize<T: serde::Serialize>(r: Result<T, qbz_core::CoreError>) -> Result<Value, Response<Cursor<Vec<u8>>>> {
    match r {
        Ok(v) => Ok(serde_json::to_value(v).unwrap_or(Value::Null)),
        Err(_) => Err(err_json(502, "discover_failed", "discover request to Qobuz failed", "try again in a moment")),
    }
}

fn auth_gate(state: &ApiState) -> Option<Response<Cursor<Vec<u8>>>> {
    let needs_auth = state
        .shared
        .lock()
        .map(|s| s.auth == AuthState::NeedsAuth)
        .unwrap_or(false);
    if needs_auth {
        Some(err_json(409, "needs_auth", "not logged in to Qobuz", "run: qbzd login"))
    } else {
        None
    }
}
