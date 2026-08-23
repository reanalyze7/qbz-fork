// crates/qbzd/src/api/search/ — GET /api/search (02-cli-and-api.md §2.3/§3.4
// row 19, P1). The first "originate playback" read verb: qbzd can now FIND
// music by name, not only receive it via QConnect or take track ids by hand.
//
// Server-side shaping (stable, typed contract): typed searches return the
// core's `SearchResultsPage<T>` (qbz-models/src/types.rs:811) verbatim under a
// per-category key; `type=all` runs the four typed searches and assembles them
// under one envelope. Reusing the shipped serde shapes keeps `--json` a frozen
// machine surface (§3.1.4) instead of leaking `catalog_search`'s raw Qobuz JSON.
//
// Blacklist filtering is intentionally NOT applied — the daemon opens no
// blacklist store (fail-open by design, qbz-core/src/core.rs:127-143); a
// documented GUI-parity delta (results may include items the desktop hides).
mod errors;
mod params;
#[cfg(test)]
mod tests;

use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use super::{json, ApiState};

use errors::{auth_gate, upstream_error};
use params::{parse_query, Category, SearchType};

/// `GET /api/search?q=&type=all|albums|tracks|artists|playlists&limit=&offset=`.
/// `query` is the raw query string (no leading `?`); `route()` strips it off the
/// path before dispatch. Errors: 409 `needs_auth`, 400 `bad_request` (missing
/// query / unknown type), 502 `search_failed` (upstream Qobuz error).
///
/// Auth: gates on `NeedsAuth` exactly like `queue::add` — the typed searches
/// call the Qobuz client, which is `CoreError::NotInitialized` without a session,
/// so a needs-auth daemon answers 409 (→ CLI exit 4) rather than a bare failure.
pub fn search(state: &ApiState, query: &str) -> Response<Cursor<Vec<u8>>> {
    if let Some(resp) = auth_gate(state) {
        return resp;
    }

    let params = match parse_query(query) {
        Ok(p) => p,
        Err((message, hint)) => return super::err_json(400, "bad_request", &message, &hint),
    };

    // Which categories to fetch — `all` fans out to the four typed searches;
    // a typed request populates exactly one key (the rest stay `null`).
    let (do_albums, do_tracks, do_artists, do_playlists) = match params.stype {
        SearchType::All => (true, true, true, true),
        SearchType::One(Category::Albums) => (true, false, false, false),
        SearchType::One(Category::Tracks) => (false, true, false, false),
        SearchType::One(Category::Artists) => (false, false, true, false),
        SearchType::One(Category::Playlists) => (false, false, false, true),
    };

    let mut albums = Value::Null;
    let mut tracks = Value::Null;
    let mut artists = Value::Null;
    let mut playlists = Value::Null;

    if do_albums {
        match state.rt.block_on(state.runtime.core().search_albums(
            &params.q,
            params.limit,
            params.offset,
            None,
        )) {
            Ok(page) => albums = serde_json::to_value(page).unwrap_or(Value::Null),
            Err(_) => return upstream_error(),
        }
    }
    if do_tracks {
        match state.rt.block_on(state.runtime.core().search_tracks(
            &params.q,
            params.limit,
            params.offset,
            None,
        )) {
            Ok(page) => tracks = serde_json::to_value(page).unwrap_or(Value::Null),
            Err(_) => return upstream_error(),
        }
    }
    if do_artists {
        match state.rt.block_on(state.runtime.core().search_artists(
            &params.q,
            params.limit,
            params.offset,
            None,
        )) {
            Ok(page) => artists = serde_json::to_value(page).unwrap_or(Value::Null),
            Err(_) => return upstream_error(),
        }
    }
    if do_playlists {
        match state.rt.block_on(state.runtime.core().search_playlists(
            &params.q,
            params.limit,
            params.offset,
        )) {
            Ok(page) => playlists = serde_json::to_value(page).unwrap_or(Value::Null),
            Err(_) => return upstream_error(),
        }
    }

    json(
        200,
        serde_json::json!({
            "query": params.q,
            "type": params.stype.as_str(),
            "limit": params.limit,
            "offset": params.offset,
            "albums": albums,
            "tracks": tracks,
            "artists": artists,
            "playlists": playlists,
        }),
    )
}
