# crates/qbz-playlist-import/src/providers/apple.rs (327 lines)

Apple Music playlist import: URL detection/parsing, HTML scrape (og:meta +
embedded JSON), track extraction.

## Proposed split

- `apple/mod.rs` (~90 lines) — re-export surface: `detect_resource`,
  `parse_playlist_id`, `fetch_playlist` (the public API 3 fns; keep
  `fetch_playlist`'s body here since it's the main orchestration, or move
  to `scrape.rs` below if `mod.rs` still runs long).
- `apple/parse_url.rs` (~50 lines) — `detect_resource`, `parse_playlist_id`
  (pure URL parsing, no I/O).
- `apple/scrape.rs` (~110 lines) — `fetch_playlist` (HTTP + assembly),
  `extract_script`, `find_track_items`, `extract_meta`, `unescape_basic`
  (the HTML/JSON scraping helpers).
- `apple/tests.rs` (~115 lines) — existing test module.

## Tricky coupling

- `fetch_playlist` uses `crate::http::http()`, `crate::errors::
  PlaylistImportError`, `crate::models::{ImportPlaylist, ImportProvider,
  ImportTrack}` — unchanged imports regardless of split.
- Tests reference `super::super::MusicResource` (from the parent
  `providers` module) — keep that relative path working by preserving the
  `providers::apple::{parse_url,scrape}` nesting (two levels under
  `providers`, not sibling to it).

## Verify after split

`cargo build -p qbz-playlist-import`, `cargo test -p qbz-playlist-import
providers::apple::` (9 existing tests).
