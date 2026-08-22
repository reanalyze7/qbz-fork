# crates/qbz-music-link/src/qobuz_search.rs (161 lines)

## Summary
Progressive-fallback Qobuz search (`search_qobuz_smart`): full query -> cleaned-title
query -> artist-only query, used by the link resolver to find a Qobuz match for a
cross-platform link; also defines `MusicLinkResult` and the `clean_title` helper.

## Proposed split
This file is only 31 lines over budget; a light split is enough — no need to
fragment search attempts across many tiny files.

- `qobuz_search/mod.rs` (~25 lines) — re-exports `MusicLinkResult`, `search_qobuz_smart`,
  `clean_title`; houses the `MusicLinkResult` enum (pure data, currently lines 11-24).
- `qobuz_search/search.rs` (~115 lines) — `search_qobuz_smart` and its three attempts
  (full/cleaned/artist-only), unchanged logic, `use super::MusicLinkResult`.
- `qobuz_search/title.rs` (~25 lines) — `clean_title` (pure string transform), easy to
  unit-test in isolation later.

## Re-export surface
`qobuz_search/mod.rs` re-exports everything so `crate::qobuz_search::{MusicLinkResult,
search_qobuz_smart}` (used from `detection.rs`/wherever the link resolver calls in)
keeps working unchanged.

## Coupling / watch out
- `search_qobuz_smart` is `pub(crate)`, not `pub` — keep that visibility across the
  split so it doesn't leak outside the crate.
- Depends on `crate::bridge::QobuzSearchBridge` and `crate::errors::MusicLinkError` —
  no change needed, just re-import in `search.rs`.
- `clean_title` is called both by `search.rs` and (per the module doc) mirrors
  `src-tauri`'s original; keep the doc comment referencing the port source.

## Verify after split
- `cargo check -p qbz-music-link`
- `cargo test -p qbz-music-link qobuz_search` (no existing tests in this file today,
  but check the crate's test suite for callers)
- Grep for `qobuz_search::` importers in `qbz-music-link` and any GUI/CLI crates using
  the music-link resolver, confirm they still compile.
