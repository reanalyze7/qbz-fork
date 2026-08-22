# crates/qbz-playlist-import/src/providers/spotify.rs (299 lines)

## Summary
Spotify playlist-import provider: URL/URI resource detection, playlist-ID
parsing, and (since the client_credentials API is gone) an embed-page HTML
scraper that extracts `__NEXT_DATA__` JSON to build track/artist metadata and
full playlist imports, plus its own test suite.

## Proposed split
- `spotify/mod.rs` (~50 lines) — module doc (lines 1-5), `detect_resource`
  and `parse_playlist_id` (lines 14-70) — the URL/URI recognition surface
  other provider-dispatch code calls first, plus `#[cfg(test)] mod tests;`
  declaration.
- `spotify/embed.rs` (~90 lines) — `fetch_embed_metadata`,
  `fetch_playlist`, `fetch_playlist_from_embed` (lines 72-216) — the async
  HTTP + embed-scraping fetch logic, the actual provider behavior.
- `spotify/html.rs` (~15 lines) — `extract_script` (lines 218-224) — the
  tiny shared HTML-substring extractor both `fetch_embed_metadata` and
  `fetch_playlist_from_embed` use.
- `spotify/tests.rs` (~75 lines) — the entire `#[cfg(test)] mod tests` block
  (lines 226-299): `parse_playlist_id_table`, `detect_resource_track_album_playlist`,
  `extract_script_pulls_next_data_payload`, `extract_script_missing_id_is_none`.

## Re-export surface
`spotify/mod.rs` re-exports `detect_resource`, `parse_playlist_id`,
`fetch_embed_metadata`, `fetch_playlist` at
`crate::providers::spotify::*` — the provider-dispatch table (in
`providers/mod.rs` or wherever `MusicResource`/`ImportProvider::Spotify` is
matched) keeps calling `spotify::detect_resource(...)` and
`spotify::fetch_playlist(...)` unchanged. `spotify.rs` becomes
`spotify/mod.rs`.

## Coupling / watch out
- `extract_script` is called from both `fetch_embed_metadata` (in
  `embed.rs`) and `fetch_playlist_from_embed` (also `embed.rs`) — if kept in
  a separate `html.rs`, it needs `pub(super)` or `pub(crate)` visibility
  (currently a private top-level `fn`). Simpler alternative: keep
  `extract_script` inline in `embed.rs` since it's only 7 lines and only
  used there — this avoids a visibility bump for a near-trivial helper; only
  extract to `html.rs` if another provider also wants URL-agnostic
  `<script id="...">` extraction (grep other providers first).
- `detect_resource` references `super::MusicResource` /
  `super::MusicProvider` (the parent `providers` module's shared enum) —
  this relative path (`super::`) is unaffected by turning `spotify.rs` into
  `spotify/mod.rs` (still one level under `providers/`).
- Tests reference `super::super::MusicResource` /
  `super::super::MusicProvider` (two levels up, since tests are nested
  inside `mod tests` inside `spotify.rs` inside `providers/`) — when moved to
  `spotify/tests.rs` with `use super::*;`, the path becomes
  `super::MusicResource` (one `super` less, since `tests.rs` is a sibling
  module of `mod.rs`, not nested inside it) — double check this path
  adjustment carefully, it's the easiest thing to get subtly wrong in this
  split.
- The Spotify API-deprecation note (module doc, lines 2-5: "As of 2026-03-06,
  client_credentials is no longer available… embed-only, ~50 track limit, no
  ISRC/album data") is important operational context — keep it in
  `spotify/mod.rs`'s module doc, not buried in `embed.rs`.

## Verify after split
- `cargo test -p qbz-playlist-import providers::spotify::` — all 4 existing
  tests green.
- `cargo check -p qbz-playlist-import` and grep for `providers::spotify::` /
  `spotify::detect_resource` / `spotify::fetch_playlist` importers (the
  provider-dispatch table) to confirm the public path is unchanged.
