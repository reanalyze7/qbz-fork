# crates/qbz-music-link/src/lib.rs (183 lines)

## Summary
Crate root for the frontend-agnostic cross-platform music-link resolver:
declares the crate's submodules (`bridge`/`detection`/`errors`/`fast_path`/
`odesli`/`qobuz_search`), re-exports their public API, and hosts the two
top-level orchestration functions `resolve_music_link` (the public entry
point: native Qobuz fast-path → resource detection → dispatch) and
`resolve_via_odesli_and_search` + `fetch_metadata_via_odesli` (the non-Qobuz
path: platform-metadata fast-path → Odesli fallback → Qobuz search).

## Proposed split
This file is already mostly a thin crate-root (`mod`/`pub use` declarations
take up lines 15–32); the only thing pushing it over 130 lines is the 3
orchestration functions (lines 43–183, ~140 lines). Split those into a new
submodule, keeping `lib.rs` as pure crate-root plumbing:

- `lib.rs` (~35 lines) — module doc, `mod` declarations, `pub use` re-exports
  (lines 1–34), plus `pub use resolve::resolve_music_link;` for the new
  submodule below.
- `resolve.rs` (~145 lines) — the 3 functions: `resolve_music_link` (lines
  43–100, the public entry point), `resolve_via_odesli_and_search` (lines
  107–158), `fetch_metadata_via_odesli` (lines 161–183). Still likely a
  touch over 130 depending on final formatting — if so, split
  `fetch_metadata_via_odesli` (the Odesli-fetch-with-retry helper, ~25 lines)
  into `resolve/odesli_fetch.rs`, or split `resolve_via_odesli_and_search`
  out into its own `resolve/via_odesli.rs` (~55 lines) leaving `resolve.rs`
  with just the public `resolve_music_link` entry point (~60 lines).

## Re-export surface
`lib.rs` stays the crate root and the only re-export surface — `pub use
resolve::resolve_music_link;` keeps `qbz_music_link::resolve_music_link` at
the same path every caller (Tauri/Slint/TUI frontends) already uses. The
existing `pub use bridge::QobuzSearchBridge;` etc. lines are untouched.

## Coupling / watch out
- `resolve_music_link` calls `detection::MusicProvider`/`MusicResource`
  (aliased as `Provider` via `use detection::MusicProvider as Provider;` at
  line 34) — this alias needs to move with the function into `resolve.rs`.
- `resolve_via_odesli_and_search` calls both `fast_path::try_direct_platform_
  metadata` and (via `fetch_metadata_via_odesli`) `qobuz_search::search_qobuz_
  smart` — if these 3 functions get split across multiple new files, each
  needs the right `use crate::{fast_path, qobuz_search, odesli::ContentType};`
  imports; keeping all 3 in one `resolve.rs` avoids this entirely, so only
  split further if line count truly requires it.
- The crate's public API contract (`QobuzSearchBridge`, `MusicLinkResult`,
  `MusicLinkError`, `SongLinkClient`/`ContentType`, `resolve_link`/
  `LinkResolverError`/`ResolvedLink` from `qbz_qobuz`) must all still resolve
  at `qbz_music_link::*` — this split touches none of those re-export lines,
  only moves the 3 orchestration functions, so risk is low.

## Verify after split
- `cargo build -p qbz-music-link`.
- `cargo test -p qbz-music-link` — check for existing tests exercising
  `resolve_music_link`/`resolve_via_odesli_and_search`.
- `cargo check` on downstream crates (Tauri/Slint/TUI frontends) that call
  `qbz_music_link::resolve_music_link` to confirm the public path is
  unaffected.
