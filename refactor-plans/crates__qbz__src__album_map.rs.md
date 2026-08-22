# crates/qbz/src/album_map.rs (264 lines)

## Summary
Shared album → card mapping used by every Qobuz album surface (label
releases, favorites, toolbar-driven album lists): decodes the V2-nested
Qobuz album shape with flat-field fallback into a plain `AlbumCard`, then a
Slint `AlbumCardItem`; also owns quality-tier classification and the local
album-list sort used by grid/list toolbar views.

## Proposed split
Split by responsibility (decode/map ↔ quality classification ↔ Slint-item
conversion ↔ sort) into a `album_map/` directory:

- `album_map/mod.rs` (~50 lines) — module doc, `AlbumCard` struct
  definition (the plain data type shared by every sub-module),
  `format_album_title`, and `pub use` re-exports of everything below.
- `album_map/map.rs` (~110 lines) — `map_album` (the main V2-nested/flat
  decode), `album_artist` (artist name/id resolution helper it calls).
- `album_map/quality.rs` (~40 lines) — `tier`, `tier_hires`,
  `release_type_label`, `classify_release_type` (the quality-tier and
  release-type classification helpers, all small pure functions grouped by
  "how do we label this album").
- `album_map/to_item.rs` (~40 lines) — `to_item` (the `AlbumCard` →
  `AlbumCardItem` Slint conversion, including the favorite/pin state
  lookups against `crate::local_favorites`/`crate::fav_cache`/
  `crate::pinned`).
- `album_map/sort.rs` (~25 lines) — `sort_album_items`.

Given the file is only 264 lines, `map.rs` + `quality.rs` could also be
merged into one ~150-line file if the split feels over-fragmented — but
since 150 > 130, keeping them separate as proposed is the straightforward
compliant option.

## Re-export surface
`album_map/mod.rs` becomes the target of the existing `mod album_map;` (or
`pub mod album_map;`) in `crates/qbz/src/lib.rs` (or `main.rs`). It must
`pub use map::*; pub use quality::*; pub use to_item::*; pub use sort::*;`
so every current caller path (`crate::album_map::map_album`,
`crate::album_map::AlbumCard`, `crate::album_map::to_item`,
`crate::album_map::sort_album_items`, `crate::album_map::tier`, etc. — the
doc comment confirms both `label.rs` and `favorites.rs` call through here)
keeps resolving unchanged.

## Coupling / watch out
- `AlbumCard` (the struct) is constructed in `map.rs`'s `map_album` and
  consumed in `to_item.rs`'s `to_item` — both files need `use
  super::AlbumCard;` if the struct stays in `mod.rs`, or `map.rs` needs to
  own it and `to_item.rs` needs `use super::map::AlbumCard;`. Proposed
  layout above keeps it in `mod.rs` to avoid this ambiguity.
- `to_item` reaches into `crate::local_favorites::is_favorite`,
  `crate::fav_cache::is_album_favorite`, and `crate::pinned::is_pinned` —
  these are cross-cutting app-level stores; no special handling needed
  beyond normal `use crate::...` in `to_item.rs`.
- `qbz_i18n::t`/`qbz_i18n::mark` calls in `quality.rs`'s
  `release_type_label`/`classify_release_type` — note the doc comment's
  careful split between "mark at definition, translate at call site" (for
  the i18n string extractor to see English literals) — preserve this
  exact `mark`/`t` pairing when moving the functions, don't accidentally
  wrap `classify_release_type`'s return value in `t()` twice.
- `crate::dates::release_label` and `crate::AlbumCardItem` (the Slint
  generated type) are the two external crate-level deps used respectively
  by `map.rs` and `to_item.rs`.

## Verify after split
- `cargo build -p qbz` and `cargo test -p qbz album_map::` (check for any
  `#[cfg(test)]` block — none observed in this read; re-grep after the
  real split in case tests exist elsewhere referencing `album_map`
  internals directly, e.g. in an integration test file).
- Smoke-test in the running app: open a Label page, Favorites → Albums, and
  any toolbar album grid; verify quality tier badges, release-type
  (Album/EP/Single/Live/Compilation) labels, favorite/pin heart+badge
  state, and every sort option (newest/oldest/title/artist asc+desc) still
  work identically.
