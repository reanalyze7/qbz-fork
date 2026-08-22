# crates/qbz-playlist-import/src/providers/mod.rs (221 lines)

## Summary
Cross-provider glue for the playlist importer: URL-to-resource detection
(`detect_music_resource`, `detect_provider`), the `MusicProvider` /
`MusicResource` / `ProviderKind` enums, and the dispatching `fetch_playlist`
that calls into the per-platform submodules (`apple`, `deezer`, `spotify`,
`tidal`).

## Proposed split
Mostly over budget because of the inline `#[cfg(test)] mod tests` (~85
lines) that table-tests every provider's detection. Split tests out; keep the
small amount of production logic together since it's already cohesive
(detection + dispatch, no pure/IO seam worth forcing):

- `providers/mod.rs` (~140 lines) — module doc, submodule declarations,
  `MusicProvider`, `MusicResource`, `ProviderKind` enums,
  `detect_music_resource`, `detect_provider`, `fetch_playlist`.
- `providers/tests.rs` (~85 lines) — the entire `#[cfg(test)] mod tests`
  block (table test + song-link/playlist-routing cases), included via
  `#[cfg(test)] mod tests;` at the bottom of `mod.rs`, referencing items
  through `super::*`.

## Re-export surface
No change needed — `mod.rs` already IS the public surface
(`qbz_playlist_import::providers::{detect_provider, fetch_playlist, ...}`);
splitting only the test module doesn't touch any public path.

## Coupling / watch out
- `detect_music_resource` and `detect_provider` both loop over the same 4
  submodules (`spotify`, `apple`, `tidal`, `deezer`) in the same order —
  keep that order consistent if either function is touched later, since
  tests assert on which provider wins ambiguous URLs.
- `ProviderKind::Tidal` hardcodes the default storefront/country (`None` ->
  US) with a comment explaining the Tauri original read an env var — flag
  this as a "don't lose this comment" spot when moving code.

## Verify after split
- `cargo test -p qbz-playlist-import providers` — all detection table tests
  green.
- `cargo check -p qbz-playlist-import` for downstream callers of
  `providers::fetch_playlist` / `detect_provider`.
