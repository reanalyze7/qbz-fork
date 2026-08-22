# crates/qbz-integrations/src/musicbrainz/location.rs (315 lines)

## Summary
Part of the MusicBrainz "location-based artist discovery / scene" pipeline:
extracts artist metadata + resolves location precision (city/state/country)
from MB area data, computes an affinity score for scene-candidate ranking,
builds a scene cache key, and formats MB life-span dates.

## Proposed split
By domain (location resolution / country-code table / affinity scoring /
date formatting) — these four concerns are only loosely related (they share
the module's public surface, not much internal state):

- `location/mod.rs` (~25 lines) — module doc, imports, `mod` wiring (`mod
  resolve; mod country_codes; mod affinity; mod dates;`), re-exports.
- `location/resolve.rs` (~100 lines) — `extract_metadata`,
  `resolve_location` (the begin_area/area/country-code cascade — the core
  "location-based discovery" logic named in the module doc).
- `location/country_codes.rs` (~50 lines) — `country_code_to_name` and its
  match table (currently 40+ arms; purely data, zero logic, cleanest
  standalone unit — also the easiest one to later replace with a real
  ISO-3166 crate if that's ever wanted).
- `location/affinity.rs` (~55 lines) — the `SCORE_*` consts and
  `compute_affinity_score`, `build_scene_cache_key` (the "candidate scoring"
  half of the pipeline).
- `location/dates.rs` (~65 lines) — `format_life_span_date`, `format_mb_date`
  (the two date-formatting helpers; `format_mb_date`'s two duplicate 12-arm
  month-name matches for the `2` and `3` parts-length cases are a pre-
  existing dedup opportunity worth a one-line note but out of scope for a
  pure line-count split).

## Re-export surface
`location/mod.rs` — becomes
`crates/qbz-integrations/src/musicbrainz/location/mod.rs`. Keeps
`pub fn extract_metadata`, `pub fn compute_affinity_score`,
`pub fn build_scene_cache_key`, `pub fn format_life_span_date` re-exported
(via `pub use resolve::extract_metadata; pub use affinity::*; pub use
dates::format_life_span_date;`) so `super::musicbrainz::location::X` (or
however the parent `musicbrainz` module re-exports it) call sites in the
scene-discovery pipeline are unaffected. `resolve_location`,
`country_code_to_name`, and `format_mb_date` stay private (only used
internally).

## Coupling / watch out
- `resolve_location` (in `resolve.rs`) calls `country_code_to_name` (proposed
  for `country_codes.rs`) in three of its four branches — this is the one
  real cross-file dependency in the split; keep `country_code_to_name` `pub(
  super)` or `pub(crate)` so `resolve.rs` can reach it via
  `super::country_codes::country_code_to_name`.
- `extract_metadata` (resolve.rs) calls `extract_affinity_seeds` and
  `normalize_genre` from the SIBLING `super::genre` module (imported at the
  top of the current file via `use super::genre::{...}`) — this dependency
  is unaffected by the split (it's already a cross-module import today), just
  make sure the `use super::genre::...` line is repeated in `resolve.rs`
  after the split, not left orphaned in a file that no longer needs it.
- `compute_affinity_score` (affinity.rs) also calls `normalize_genre` from
  `super::genre` — same note, needs its own `use` after the split.
- The four submodules (`resolve`, `country_codes`, `affinity`, `dates`) don't
  otherwise call each other — this is a genuinely low-risk, mostly-mechanical
  split once the two `country_code_to_name` and two `genre::` call sites
  above are wired correctly.
- Shared types (`AffinitySeeds`, `Area`, `ArtistFullResponse`,
  `ArtistLocation`, `ArtistMetadata`, `ArtistType`, `LifeSpan`,
  `LocationPrecision`) come from `super::{...}` (the parent `musicbrainz`
  module) — repeat the relevant subset of that `use super::{...}` import in
  whichever file uses each type (e.g. `resolve.rs` needs `Area`,
  `ArtistFullResponse`, `ArtistLocation`, `ArtistMetadata`, `ArtistType`,
  `LocationPrecision`; `dates.rs` needs only `LifeSpan`).

## Verify after split
- `cargo check -p qbz-integrations` (or the relevant crate name) for the
  crate itself and any caller of
  `musicbrainz::location::{extract_metadata, compute_affinity_score,
  build_scene_cache_key, format_life_span_date}`.
- No `#[cfg(test)]` block exists in this file today — if the scene-discovery
  pipeline has integration tests elsewhere in the crate that exercise this
  module indirectly, run those (`cargo test -p qbz-integrations`) to confirm
  behavior is unchanged; otherwise this split has no direct unit-test safety
  net and should be double-checked by re-reading the diff carefully for the
  two cross-file call sites noted above.
