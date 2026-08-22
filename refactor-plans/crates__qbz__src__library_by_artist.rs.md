# crates/qbz/src/library_by_artist.rs (134 lines)

## Summary
In-memory per-artist library index (favorites grouped by artist id) for the
ArtistPage catalog/library toggle, seeded once per session. Only 4 lines
over budget.

## Proposed split
Barely over — trim by moving the Slint-mapping helper out:

- `library_by_artist/mod.rs` (~95 lines) — `ArtistLibrary` struct, `INDEX`
  static, `seed`, `get`, `pub use` of `view`.
- `library_by_artist/view.rs` (~40 lines) — `album_items`, `track_items`
  (the two Slint `ModelRc` builders).

## Re-export surface
`library_by_artist/mod.rs` stays the `mod library_by_artist;` target; `pub
use view::*;` keeps `album_items`/`track_items` at
`crate::library_by_artist::X` (used by `label.rs`'s
`apply_label_library`, per that file's `crate::library_by_artist::
track_items`/`album_items` calls).

## Coupling / watch out
- `seed()` calls `crate::library_by_label::seed_from_parts(&fav_tracks,
  &fav_albums)` inline — a cross-cutting side effect that feeds a sibling
  module's index from the SAME fetch (explicitly commented "no doubled
  pagination"). Keep this call in `mod.rs`'s `seed`, do not move it into
  `view.rs`.
- `track_items` duplicates most of `favorites::apply_favorites`'s
  `TrackItem` field mapping (all favorites, `is_favorite: true` always) —
  flag as pre-existing duplication, not something to fix in a mechanical
  split.
- Given this file is barely over budget, a simpler alternative is just
  inlining `album_items`/`track_items` calls elsewhere if a reviewer prefers
  not to add a submodule for 4 lines — flagging both options.

## Verify after split
- `cargo build -p qbz` (no `#[cfg(test)]` in this file).
- Smoke-test the ArtistPage catalog/library toggle after login (index seeds
  at session start) to confirm favorite tracks/albums group correctly.
