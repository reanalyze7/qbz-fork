# crates/qbz-music-link/src/detection.rs (392 lines)

## Summary
Music-URL detection: `detect_music_resource()` dispatches a URL to Qobuz / song.link
/ per-provider (Spotify, Apple Music, Tidal, Deezer) detection, each provider living
in its own inline `pub mod` with `detect_resource` + a `parse_playlist_id` helper;
also defines the shared `MusicProvider`/`MusicResource` enums and Spotify's
embed-metadata scrape.

## Proposed split
This file is already internally organized by provider (`mod spotify { ... }`, `mod
apple { ... }`, etc., each with a `// ── Provider ──` banner) — the split is
mechanical: promote each inline `mod` to its own file, by domain (one file per
streaming provider), which is exactly the "one package per domain" principle.

- `detection/mod.rs` (~55 lines) — module doc, `MusicProvider`/`MusicResource` enum
  definitions (shared data types, currently lines 8-39), `detect_music_resource()`
  dispatcher (lines 41-79), `mod` declarations for the four provider submodules.
- `detection/spotify.rs` (~125 lines) — the `spotify` module verbatim (detect_resource,
  parse_playlist_id, fetch_embed_metadata + its `extract_script` helper) — the
  largest provider due to the embed-metadata HTTP scrape.
- `detection/apple.rs` (~60 lines) — the `apple` module verbatim (detect_resource,
  parse_playlist_id).
- `detection/tidal.rs` (~60 lines) — the `tidal` module verbatim.
- `detection/deezer.rs` (~60 lines) — the `deezer` module verbatim.

## Re-export surface
`detection/mod.rs` keeps `pub mod spotify; pub mod apple; pub mod tidal; pub mod
deezer;` (already `pub mod` today, so callers using `crate::detection::spotify::
parse_playlist_id` etc. keep working) and re-exports `MusicProvider`, `MusicResource`,
`detect_music_resource` at `crate::detection::*` unchanged.

## Coupling / watch out
- Each provider module currently does `use super::*;` to reach `MusicProvider`/
  `MusicResource` — after the split, change to `use crate::detection::{MusicProvider,
  MusicResource};` (or `use super::{MusicProvider, MusicResource};` if `mod.rs` is the
  direct parent) in each new file.
- `spotify::fetch_embed_metadata` is the only async/network function in this file
  (uses `reqwest::get`) — it's a clear IO-vs-pure boundary already isolated inside
  `spotify.rs`; could be split further into `spotify/detect.rs` (pure) +
  `spotify/embed.rs` (IO) later if `spotify.rs` alone still feels big, but 125 lines
  fits under budget as one file.
- `detect_music_resource()`'s ordering (Qobuz first, then song.link, then the four
  providers in a fixed order) is significant — preserve call order exactly in
  `mod.rs`.

## Verify after split
- `cargo check -p qbz-music-link`
- `cargo test -p qbz-music-link` (no `#[cfg(test)]` block exists in this file today —
  confirm sibling test files, if any, still resolve `detection::{spotify,apple,tidal,
  deezer}::*` paths).
- Grep the `qbz-music-link` crate and any UI/CLI crate for `detection::` usages to
  confirm the public paths are unaffected.
