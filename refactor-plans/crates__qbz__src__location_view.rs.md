# crates/qbz/src/location_view.rs (164 lines)

## Summary
`ArtistsByLocationView` controller: loads a page of scene-discovery artist
candidates (MusicBrainz + Qobuz), maps them to grid items, and pushes into
`LocationViewState` (load/append/reset), plus artwork job derivation.

## Proposed split
Only ~34 lines over — split async data loading from pure view-state pushing:

- `location_view/mod.rs` (~55 lines) — `LocationData`, `ArtistCard` structs,
  `PAGE_SIZE`, `map_candidate`, `pub use` of `load` and `view` submodules.
- `location_view/load.rs` (~65 lines) — `load_scene` (async core-call +
  blacklist filtering).
- `location_view/view.rs` (~55 lines) — `to_item`, `apply_scene`,
  `append_scene`, `reset_scene`, `artwork_jobs` (pure Slint-state pushing,
  no async).

## Re-export surface
`location_view/mod.rs` stays the `mod location_view;` target. Public fns
used by the artist-location screen (`load_scene`, `apply_scene`,
`append_scene`, `reset_scene`, `artwork_jobs`) stay reachable via `pub use
load::load_scene; pub use view::*;`.

## Coupling / watch out
- `to_item` (view.rs) calls `crate::fav_cache::is_artist_favorite` and
  `crate::pinned::is_pinned` — no change, just keep those `crate::` imports.
- `load_scene`'s blacklist re-filter comment (D-FIX-c) is load-bearing
  documentation explaining why filtering happens on every call, not cached
  — keep the comment attached to the filtering code when moved.
- `map_candidate` is used only by `load_scene` but lives in `mod.rs` per
  this plan for grouping with the structs it builds — either location is
  fine; if moved to `load.rs` instead, no other caller needs it.

## Verify after split
- `cargo build -p qbz` (no `#[cfg(test)]` in this file).
- Smoke-test "Artists by location" scene discovery screen: initial load,
  "load more" pagination, and artwork loading for the grid.
