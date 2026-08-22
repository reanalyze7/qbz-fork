# crates/qbz-models/src/source.rs (270 lines)

## Summary
Source-aware playback types shared across all frontends: `PlaybackSource`
(Qobuz/OfflineCache/Local) and `TrackOriginTag` (adds `ExternalUnknown` for
the strict Qobuz-Connect cast-admission gate), `ArtworkRef` (uniform cover-art
origin), and a `QueueTrack` extension trait-like `impl` providing
`source_kind()`/`artwork_ref()`.

## Proposed split
By type (playback-source enums / artwork-ref / QueueTrack glue), production
code is only ~183 lines so this is a light split, mostly to make room under
130 lines per file plus keep the sizeable test module separate:

- `source/mod.rs` (~15 lines) — module doc, imports, `mod` wiring (`mod
  playback_source; mod artwork_ref; mod queue_track_ext;`), re-exports.
- `source/playback_source.rs` (~90 lines) — `PlaybackSource` enum + its impl
  (`from_source_str`, `as_source_str`, `is_qobuz_streamable`,
  `is_castable_to_qconnect`, `from_source_str_strict`), plus `TrackOriginTag`
  enum + its impl (`is_castable_to_qconnect`). These two types are tightly
  coupled (the strict-parse constructor lives on `PlaybackSource` but returns
  `TrackOriginTag`) so keep them in one file rather than splitting further.
- `source/artwork_ref.rs` (~55 lines) — `ArtworkRef` enum + its impl
  (`is_empty`, `to_mpris_url`).
- `source/queue_track_ext.rs` (~30 lines) — the `impl QueueTrack` block
  adding `source_kind()` and `artwork_ref()`; needs `use crate::playback::
  QueueTrack;` plus the two sibling modules' types.
- `source/tests.rs` (~90 lines) — the `#[cfg(test)] mod tests` block, wired
  via `#[cfg(test)] mod tests;` in `mod.rs`.

## Re-export surface
`source/mod.rs` — becomes `crates/qbz-models/src/source/mod.rs`. Keeps
`pub enum PlaybackSource`, `pub enum TrackOriginTag`, `pub enum ArtworkRef`
re-exported at `crate::source::{PlaybackSource, TrackOriginTag, ArtworkRef}`
(via `pub use playback_source::{PlaybackSource, TrackOriginTag}; pub use
artwork_ref::ArtworkRef;`) so every frontend (`qbz`, `qbz-player`, `qbzd`,
etc.) that does `use qbz_models::source::PlaybackSource` (or however
`qbz-models` re-exports its top-level `source` module — check `lib.rs` for
`pub mod source;` or `pub use source::*;`) is unaffected. The
`impl QueueTrack` block in `queue_track_ext.rs` needs no re-export — inherent
methods are found via the type itself once the `impl` is anywhere in the
crate.

## Coupling / watch out
- `PlaybackSource::from_source_str_strict` is defined in a SECOND, separate
  `impl PlaybackSource` block (lines 84-97) distinct from the first `impl
  PlaybackSource` block (lines 30-69) purely because it returns
  `TrackOriginTag` (defined in between, at line 76) — when splitting, these
  two `impl PlaybackSource` blocks can be merged into one in
  `playback_source.rs` since Rust doesn't require the split (the original
  split was likely just documentation-comment structuring, not a language
  requirement).
- `TrackOriginTag::is_castable_to_qconnect` and
  `PlaybackSource::is_castable_to_qconnect` are two SEPARATE predicates with
  the same name and semantics but on different types (used by different
  gate call-sites: `PlaybackSource` for internal queue reasoning,
  `TrackOriginTag` for the strict external-admission gate) — keep both, keep
  their distinguishing doc comments intact (they explicitly cross-reference
  each other), don't let a "just one castable predicate" simplification creep
  in during the split.
- `queue_track_ext.rs`'s `impl QueueTrack` depends on `crate::playback::
  QueueTrack` (an EXTERNAL type to this module, defined elsewhere in
  `qbz-models`) plus both `PlaybackSource` and `ArtworkRef` from its sibling
  modules — three-way dependency, but all one-directional (nothing in
  `playback_source.rs`/`artwork_ref.rs` depends back on `queue_track_ext.rs`
  or on `QueueTrack`), so no cycle risk.
- Test module exercises all three areas (source-str roundtrip, strict-parse,
  artwork-ref classification) plus constructs a full `QueueTrack` fixture
  (`track_with` helper) — keep tests together in one `tests.rs` (don't split
  by type) since `track_with` is shared across the `artwork_ref_classifies_by_value`
  tests and the roundtrip tests reference both `PlaybackSource` and
  `TrackOriginTag` together.

## Verify after split
- `cargo test -p qbz-models source` — all 5 existing tests green.
- `cargo check` across the workspace (this is a low-level shared-models
  crate depended on by nearly every frontend/backend crate) —
  `cargo check --workspace` is worth running here specifically, not just
  `-p qbz-models`, given how widely `PlaybackSource`/`ArtworkRef` are likely
  imported (grep for `qbz_models::source::` or `use qbz_models::{..., source,
  ...}` across crates before finalizing the split, and again after to diff
  broken imports).
