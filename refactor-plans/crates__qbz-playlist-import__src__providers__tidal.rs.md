# crates/qbz-playlist-import/src/providers/tidal.rs (469 lines)

## Summary
Tidal playlist import provider (OpenAPI v2): URL detection/parsing, app-token fetch,
paginated playlist-items fetch, chunked track-detail fetch with a hand-rolled JSON
`included`/`relationships` resolver, and an ISO-8601 duration parser — plus its unit
tests.

## Proposed split
By pipeline stage (URL parsing → auth → fetch pipeline → JSON mapping → tests) —
this mirrors an IO/pure split reasonably well since `parse_duration_ms` and URL
detection are pure, while the rest is network I/O.

- `providers/tidal/mod.rs` (~35 lines) — module doc, constants (`RATE_LIMIT_DELAY_MS`,
  `TIDAL_API_BASE`, `DEFAULT_COUNTRY_CODE`), `pub use` re-exports of
  `detect_resource`, `parse_playlist_id`, `fetch_playlist` so
  `crate::providers::tidal::X` paths are unchanged.
- `providers/tidal/url.rs` (~55 lines) — `detect_resource`, `parse_playlist_id` (pure
  URL parsing, no I/O).
- `providers/tidal/auth.rs` (~25 lines) — `get_app_token`.
- `providers/tidal/playlist.rs` (~65 lines) — `fetch_playlist` (the top-level
  orchestration: token → metadata fetch → track-ids → tracks → `ImportPlaylist`).
- `providers/tidal/track_ids.rs` (~70 lines) — `fetch_track_ids` (the paginated
  `relationships/items` walk).
- `providers/tidal/tracks.rs` (~150 lines) — `fetch_tracks_by_ids` (chunked fetch +
  the `included` artist/album map + per-track JSON extraction). Still over budget at
  ~150 lines; split further into:
  - `providers/tidal/tracks.rs` (~70 lines) — the chunking loop + HTTP call +
    delegation to helpers below.
  - `providers/tidal/tracks_map.rs` (~90 lines) — the `included` → `artist_map`/
    `album_map` builder and the per-item `ImportTrack` extraction closures, factored
    into named helper functions (e.g. `build_included_maps`, `track_from_json`).
- `providers/tidal/duration.rs` (~40 lines) — `parse_duration_ms` (pure ISO-8601
  duration parsing).
- `providers/tidal/tests.rs` (~55 lines) — the `#[cfg(test)] mod tests` block
  (parse_playlist_id table, detect_resource, parse_duration_ms), referencing the
  split functions via `use super::super::{url::*, duration::*};` or similar.

## Re-export surface
`providers/tidal/mod.rs` re-exports `detect_resource`, `parse_playlist_id`,
`fetch_playlist` (the three items used outside this file, e.g. by
`providers/mod.rs`'s provider dispatch) so nothing importing
`crate::providers::tidal::{detect_resource, fetch_playlist}` needs to change.

## Coupling / watch out
- `fetch_playlist` (playlist.rs) calls `get_app_token` (auth.rs), `fetch_track_ids`
  (track_ids.rs) and `fetch_tracks_by_ids` (tracks.rs) — three cross-module calls;
  keep their signatures stable (they already take `&reqwest::Client`/`&str` plain
  params, no shared mutable state, so this is low-risk).
- `detect_resource` references `super::MusicResource` and `super::MusicProvider`
  (the parent `providers` module) — when moved into `providers/tidal/url.rs`, the
  path becomes `super::super::MusicResource` (one directory deeper).
- The tests reference `super::super::MusicResource` too (as
  `super::super::MusicResource::Playlist { .. }` in the current file, itself already
  one level from `providers::mod`) — recheck the exact relative path after the move.
- `crate::QBZ_PROXY_BASE` and `crate::http::{http, USER_AGENT}` are used in
  `auth.rs`/across the fetch pipeline — re-import in each file that calls them.
- `RATE_LIMIT_DELAY_MS`/`TIDAL_API_BASE`/`DEFAULT_COUNTRY_CODE` constants are used in
  multiple submodules (track_ids.rs, tracks.rs, playlist.rs, auth.rs) — keep them in
  `mod.rs` and reference via `super::RATE_LIMIT_DELAY_MS` etc., rather than
  duplicating.

## Verify after split
- `cargo test -p qbz-playlist-import tidal` — all 3 existing tests
  (`parse_playlist_id_table`, `detect_resource_track_album_playlist`,
  `parse_duration_ms_iso8601_forms`) must stay green.
- `cargo check -p qbz-playlist-import` for the provider-dispatch call site(s) in
  `providers/mod.rs` (or wherever Tidal is registered alongside Spotify/Apple Music/
  etc. providers).
- Manual/smoke test: import a real (or recorded) Tidal playlist URL end-to-end if a
  test fixture/mocked HTTP layer exists for this crate.
