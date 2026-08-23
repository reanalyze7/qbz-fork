use std::io::Cursor;

use serde_json::Value;
use tiny_http::Response;

use qbz_models::Track;
use qbz_qobuz::link_resolver::{resolve_link, ResolvedLink};

use crate::api::{err_json, ApiState};

/// Top-tracks cap when playing a whole artist — a sane "play this artist" set,
/// not their entire catalogue.
const ARTIST_TOP_LIMIT: u32 = 50;

/// What to play, after a URL (if any) has been resolved to an id kind.
pub(super) enum Selector {
    Track(u64),
    Album(String),
    Playlist(u64),
    Artist(u64),
}

/// A URL wins over the id fields (§3.4 row 23: "URL resolved server-side"),
/// resolved via the pure `resolve_link` (qbz-qobuz/src/link_resolver.rs:50).
/// Otherwise the first present id field is used.
pub(super) fn parse_selector(body: &Value) -> Result<Selector, (String, String)> {
    let hint = "body: {\"album_id\":\"...\"} | {\"track_id\":N} | {\"playlist_id\":N} | \
                {\"artist_id\":N} | {\"url\":\"https://open.qobuz.com/...\"}"
        .to_string();

    if let Some(url) = body.get("url").and_then(|v| v.as_str()) {
        return match resolve_link(url) {
            Ok(ResolvedLink::OpenAlbum(id)) => Ok(Selector::Album(id)),
            Ok(ResolvedLink::OpenTrack(id)) => Ok(Selector::Track(id)),
            Ok(ResolvedLink::OpenArtist(id)) => Ok(Selector::Artist(id)),
            Ok(ResolvedLink::OpenPlaylist(id)) => Ok(Selector::Playlist(id)),
            Err(_) => Err((
                format!("unrecognized Qobuz URL: {url}"),
                "expected an open.qobuz.com album/track/artist/playlist link".into(),
            )),
        };
    }
    if let Some(id) = body.get("track_id").and_then(|v| v.as_u64()) {
        return Ok(Selector::Track(id));
    }
    if let Some(id) = body.get("album_id").and_then(|v| v.as_str()) {
        return Ok(Selector::Album(id.to_string()));
    }
    if let Some(id) = body.get("playlist_id").and_then(|v| v.as_u64()) {
        return Ok(Selector::Playlist(id));
    }
    if let Some(id) = body.get("artist_id").and_then(|v| v.as_u64()) {
        return Ok(Selector::Artist(id));
    }
    Err(("play requires a content selector".into(), hint))
}

/// Resolve the selector to catalog tracks + optional (context_kind, context_id)
/// provenance. A single track carries no container provenance (None).
#[allow(clippy::type_complexity)]
pub(super) fn fetch_tracks(
    state: &ApiState,
    selector: &Selector,
) -> Result<(Vec<Track>, Option<(&'static str, String)>), Response<Cursor<Vec<u8>>>> {
    match selector {
        Selector::Track(id) => match state.rt.block_on(state.runtime.core().get_tracks_batch(&[*id])) {
            Ok(tracks) => Ok((tracks, None)),
            Err(_) => Err(not_found("track", &id.to_string())),
        },
        Selector::Album(id) => match state.rt.block_on(state.runtime.core().get_album(id)) {
            Ok(album) => {
                let items = album.tracks.map(|t| t.items).unwrap_or_default();
                Ok((items, Some(("album", id.clone()))))
            }
            Err(_) => Err(not_found("album", id)),
        },
        Selector::Playlist(id) => match state.rt.block_on(state.runtime.core().get_playlist(*id)) {
            Ok(pl) => {
                let items = pl.tracks.map(|t| t.items).unwrap_or_default();
                Ok((items, Some(("playlist", id.to_string()))))
            }
            Err(_) => Err(not_found("playlist", &id.to_string())),
        },
        Selector::Artist(id) => {
            match state.rt.block_on(state.runtime.core().get_artist_tracks(*id, ARTIST_TOP_LIMIT, 0)) {
                Ok(tc) => Ok((tc.items, Some(("artist", id.to_string())))),
                Err(_) => Err(not_found("artist", &id.to_string())),
            }
        }
    }
}

fn not_found(kind: &str, id: &str) -> Response<Cursor<Vec<u8>>> {
    err_json(
        404,
        "not_found",
        &format!("{kind} {id} not found"),
        "check the id: qbzd search <QUERY>",
    )
}
