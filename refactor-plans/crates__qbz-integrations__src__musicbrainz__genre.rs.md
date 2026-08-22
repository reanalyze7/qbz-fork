# crates/qbz-integrations/src/musicbrainz/genre.rs (315 lines)

## Summary
MusicBrainz genre/tag normalization for scene discovery: static noise/broad-
tag lists, a genre-name canonicalization function, noisy/broad tag filters,
and `extract_affinity_seeds`/`genre_summary` which turn raw MB `Tag`s into an
`AffinitySeeds` struct — plus a `#[cfg(test)]` module (lines 267–315, ~50
lines).

## Proposed split
By pure/IO-adjacent concern — this whole file is pure data transformation (no
IO), so split by "static data" vs "algorithm" vs "tests":

- `musicbrainz/genre/mod.rs` (~20 lines) — module doc, `use` block, re-exports
  of `normalize_genre`, `is_broad_genre`, `extract_affinity_seeds`,
  `genre_summary`.
- `musicbrainz/genre/tables.rs` (~90 lines) — the two static arrays
  `NOISY_TAGS` (lines 11–60) and `BROAD_TAGS` (lines 66–99), plus the tiny
  `GENRE_MIN_VOTES`/`MAX_GENRES`/`MAX_TAGS` constants (lines 187–193) since
  they're config-like data alongside the tables.
- `musicbrainz/genre/normalize.rs` (~90 lines) — `normalize_genre` (lines
  102–172, the big match), `is_noisy_tag`, `is_broad_genre`.
- `musicbrainz/genre/extract.rs` (~65 lines) — `extract_affinity_seeds` (lines
  199–251) and `genre_summary` (lines 254–265), the two functions that
  actually consume `Tag`s and produce `AffinitySeeds`.
- `musicbrainz/genre/tests.rs` (~50 lines) — the existing `#[cfg(test)] mod
  tests` block (lines 267–315), referencing `super::*` (re-exported from
  `mod.rs`).

## Re-export surface
`musicbrainz/genre/mod.rs` re-exports `normalize_genre`, `is_broad_genre`,
`extract_affinity_seeds`, `genre_summary` at `crate::musicbrainz::genre::*` (or
however `mod genre;` is currently declared in `musicbrainz/mod.rs`) — no
caller changes needed since the module path (`genre`) is unchanged, only its
internal file layout becomes a directory.

## Coupling / watch out
- `super::{AffinitySeeds, Tag}` (line 8) — these types live in the PARENT
  `musicbrainz` module; every new submodule that references them
  (`extract.rs` for both, `tests.rs` for `Tag`) needs `use super::super::{...}`
  or a re-export via `genre/mod.rs`'s own `use super::{AffinitySeeds, Tag};`
  plus `pub(super) use` if submodules should reach through `super::`.
  Simplest: have `genre/mod.rs` do `use super::{AffinitySeeds, Tag};` and each
  submodule do `use super::{AffinitySeeds, Tag};` (one level up from the
  submodule = `genre/mod.rs`, which re-exports them) — verify this resolves;
  if not, submodules import directly from `crate::musicbrainz::{AffinitySeeds,
  Tag}` instead.
- `is_noisy_tag` and `is_broad_genre` both iterate their respective static
  array with a linear `.any()` scan — no shared mutable state, straightforward
  low-risk split.
- The test module currently constructs `Tag { name, count }` literals directly
  — after the split it still needs `Tag` in scope via `super::*` from
  `genre/mod.rs`, so make sure `Tag` is actually re-exported there (or import
  it directly from the grandparent module in `tests.rs`).

## Verify after split
- `cargo test -p qbz-integrations musicbrainz::genre` — all 3 existing tests
  (`test_normalize_genre`, `test_noisy_tags_filtered`, `test_empty_tags`) must
  stay green.
- `cargo check -p qbz-integrations` for any downstream crate depending on
  `qbz_integrations::musicbrainz::genre::*`.
