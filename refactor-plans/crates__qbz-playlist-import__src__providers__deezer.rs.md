# crates/qbz-playlist-import/src/providers/deezer.rs (201 lines)

## Summary
Deezer playlist-import provider: URL-shape detection (`detect_resource`,
`parse_playlist_id`) and the async Deezer API fetch (`fetch_playlist`) that
maps the raw JSON into the crate's `ImportPlaylist`/`ImportTrack` types,
plus 2 unit tests.

## Proposed split
By pure-URL-parsing vs network I/O (matches the pure/IO convention
directly):

- `deezer/mod.rs` (~10 lines) — module doc (line 1) + `pub use
  detect::{detect_resource, parse_playlist_id}; pub use fetch::fetch_playlist;`.
- `deezer/detect.rs` (~60 lines) — lines 1-69: `detect_resource`,
  `parse_playlist_id`. Pure string/URL parsing, zero I/O, easiest to unit
  test in isolation.
- `deezer/fetch.rs` (~85 lines) — lines 71-152: `fetch_playlist` (the
  async HTTP call + JSON `Value` field-by-field extraction into
  `ImportTrack`/`ImportPlaylist`).
- `deezer/tests.rs` (~50 lines) — lines 154-200: the `#[cfg(test)] mod
  tests` (`parse_playlist_id_table`, `detect_resource_track_album_playlist`)
  — both only exercise `detect.rs`'s two functions, so this can instead
  just be an inline `#[cfg(test)] mod tests` at the bottom of
  `deezer/detect.rs` rather than a separate file, since it's well under
  budget there (60 + 50 = 110 lines, fine as one file). Prefer that over a
  fifth file unless the reviewer wants a stricter 1-file-per-concern split.

## Re-export surface
`deezer/mod.rs` re-exports `detect_resource`, `parse_playlist_id`,
`fetch_playlist` so `crate::providers::deezer::detect_resource(...)` etc.
(called from `crate::providers::mod.rs`'s provider-dispatch table, e.g.
`detect_provider`/`fetch_playlist` dispatch by `MusicProvider::Deezer`)
resolve identically. This mirrors whatever sibling provider files
(`spotify.rs`, `apple_music.rs`, etc., if present in the same
`providers/` directory) already do.

## Coupling / watch out
- `super::MusicResource` and `super::MusicProvider` are referenced from
  `detect.rs` (they come from the parent `providers/mod.rs`) — after the
  split, `deezer/detect.rs` is one level deeper (`providers/deezer/
  detect.rs` instead of `providers/deezer.rs`), so `super::MusicResource`
  becomes `super::super::MusicResource` (i.e. `crate::providers::
  MusicResource`) — use an explicit `use crate::providers::{MusicResource,
  MusicProvider};` in the new file rather than relying on relative `super`
  chains, to avoid an easy-to-miss path-depth bug.
- The test module's assertions reference `super::super::MusicResource` for
  the SAME reason (line 187, `Some(super::super::MusicResource::Playlist
  {...})`) — if tests move even one directory level, this must become
  `crate::providers::MusicResource` or be recounted carefully.
- `crate::errors::PlaylistImportError`, `crate::http::http`,
  `crate::models::{ImportPlaylist, ImportProvider, ImportTrack}` are all
  used only in `fetch.rs` — self-contained, no cross-split coupling beyond
  the one extra `super::` level noted above.

## Verify after split
- `cargo check -p qbz-playlist-import` and `cargo test -p qbz-playlist-import`
  (runs the 2 existing Deezer tests, path-adjusted).
- Smoke-test an actual Deezer playlist import in the running app (paste a
  `deezer.com/playlist/...` URL into the import dialog) to confirm
  `detect_resource` + `fetch_playlist` still round-trip end-to-end, since
  the URL-parsing edge cases (locale prefix, trailing query string) are
  exactly what the unit tests cover but a live import exercises the full
  provider-dispatch wiring in `providers/mod.rs` too.
