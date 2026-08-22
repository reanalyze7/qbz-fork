# crates/qbz-music-link/src/fast_path.rs (158 lines)

## Summary
Direct-platform metadata fetchers for the music-link resolver (bypassing
Odesli for speed): Deezer/Tidal direct REST API calls and a Spotify embed-page
scrape, plus shared URL/URI entity-id extraction and a Tidal proxy-token
fetch.

## Proposed split
Only slightly over budget; split by concern — URL parsing (pure) vs. the
per-platform network calls (I/O) — which also happens to match the
pure/IO split principle directly:

- `fast_path/mod.rs` (~45 lines) — module doc, `QBZ_PROXY_BASE` const, the
  public dispatch fn `try_direct_platform_metadata` (lines 1-26); re-exports
  it as the sole public entry point.
- `fast_path/entity_id.rs` (~30 lines) — `extract_entity_id`,
  `extract_spotify_entity_id` (lines 28-52) — pure string parsing, no
  network, easy to unit test in isolation (currently untested — this split
  is also an opportunity to add tests here, though not required by this
  plan).
- `fast_path/providers.rs` (~90 lines) — `try_deezer_metadata`,
  `try_spotify_metadata`, `try_tidal_metadata`, `get_proxy_token` (lines
  54-158) — the actual network-calling implementations, one per platform.

## Re-export surface
`fast_path/mod.rs` re-exports `try_direct_platform_metadata` (the only
`pub(crate)` fn used by callers elsewhere in `qbz-music-link`, presumably
the link-resolution orchestrator) at `crate::fast_path::
try_direct_platform_metadata` — unchanged call site.

## Coupling / watch out
- `try_tidal_metadata` (providers.rs) calls `get_proxy_token` (also
  providers.rs) and `extract_entity_id` (entity_id.rs) — cross-module
  import needed, straightforward.
- `try_spotify_metadata` (providers.rs) calls
  `crate::detection::spotify::fetch_embed_metadata` — external crate-level
  dependency, unaffected by this split.
- `QBZ_PROXY_BASE` (mod.rs) is used only by `get_proxy_token`
  (providers.rs) — needs `use super::QBZ_PROXY_BASE;` or similar after the
  split.
- Given this file is only 28 lines over budget, a lighter alternative is
  simply moving the two extraction helpers (`extract_entity_id`,
  `extract_spotify_entity_id`, ~25 lines) out to a shared
  `qbz-music-link::url_utils` module if one already exists elsewhere in
  the crate for similar parsing — check for that before creating a new
  `entity_id.rs`, to avoid a redundant module.

## Verify after split
- `cargo test -p qbz-music-link fast_path` (no existing tests were found
  in this file at read time — confirm via grep before assuming none exist,
  and add basic tests for `extract_entity_id`/`extract_spotify_entity_id`
  while touching this file, per the project's "tests every time" rule).
- `cargo build -p qbz-music-link`; smoke-test resolving a Deezer/Tidal/
  Spotify link end-to-end (or via existing integration tests if the crate
  has network-mocked ones).
