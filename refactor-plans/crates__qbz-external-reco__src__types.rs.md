# crates/qbz-external-reco/src/types.rs (146 lines)

## Summary
Data types for the external-recommendations engine: the `RecoSource` enum,
raw pre-validation candidate structs (`ArtistCandidate`/`AlbumCandidate`/
`TrackCandidate`), resolved-to-Qobuz row structs (`ArtistReco`/`AlbumReco`/
`TrackReco`), the aggregate `ExternalCarousels` result, and two listening-
history snapshot structs (`LocalHistory`/`ExtHistory`).

## Proposed split
Only marginally over budget (146 vs 130). Split along the file's own two
section banners (`// ── Raw candidates ──` / `// ── Resolved rows ──`) plus
carve out the two history/aggregate structs at the bottom, matching the
by-domain principle used for `validate.rs` in this same crate.

- `types/mod.rs` (~20 lines) — module doc, `RecoSource` enum (7-16), and `pub
  use` re-exports of everything below so `crate::types::X` (and the crate's
  own `pub mod types;` re-export at the crate root, per `lib.rs`) keeps
  working unchanged.
- `types/candidates.rs` (~35 lines) — `ArtistCandidate`, `AlbumCandidate`,
  `TrackCandidate` (18-50). Pre-validation, source-tagged raw guesses.
- `types/reco.rs` (~50 lines) — `ArtistReco`, `AlbumReco`, `default_source`,
  `TrackReco` (52-95). The validated/resolved rows returned to the UI.
- `types/carousels.rs` (~30 lines) — `ExternalCarousels` (97-123). Kept
  separate from `reco.rs` since it's the aggregate/UI-facing result type, one
  level up from the individual reco rows it contains.
- `types/history.rs` (~20 lines) — `LocalHistory`, `ExtHistory` (125-146).
  The two listening-history snapshot types, unrelated to the candidate/reco
  pipeline itself — natural own file.

## Re-export surface
`types/mod.rs` is the public surface: `pub use candidates::*; pub use
reco::*; pub use carousels::ExternalCarousels; pub use history::{LocalHistory,
ExtHistory};` plus `RecoSource` defined directly in `mod.rs`. The crate's
`lib.rs` line `pub mod types;` (confirmed `pub mod types;` at line 19) needs
no change — `types/mod.rs` resolves identically to today's `types.rs`, and
every external caller uses `qbz_external_reco::types::X` paths which are
untouched.

## Coupling / watch out
- `RecoSource` is used by EVERY struct in every submodule (candidates, reco,
  and indirectly carousels via the reco types it contains) — keep it defined
  once in `mod.rs` and `use super::RecoSource;` from each sibling; do not
  duplicate the enum.
- `default_source()` (a private fn feeding `#[serde(default =
  "default_source")]` on `AlbumReco::source`) must stay in the SAME file as
  `AlbumReco` (`reco.rs`) — serde's `default = "path"` attribute resolves the
  function path relative to the struct's own module, so moving one without
  the other breaks deserialization of stored/cached `AlbumReco` JSON (this is
  exactly the kind of thing `RecoCache`/`validate.rs` serializes to disk).
- `ExternalCarousels` fields reference `ArtistReco`/`AlbumReco`/`TrackReco`
  directly (not through any trait) — `carousels.rs` needs `use super::reco::
  {ArtistReco, AlbumReco, TrackReco};`.
- This crate's OTHER `carousels.rs` file (the pipeline-building module, 28K,
  not in scope here) is a DIFFERENT file from the new `types/carousels.rs`
  proposed above — a confusing name collision. Prefer naming the new file
  `types/result.rs` or `types/aggregate.rs` instead of `types/carousels.rs`
  to avoid confusing it with the top-level `src/carousels.rs` pipeline
  module when both appear in an IDE's fuzzy-file-switcher.

## Verify after split
- `cargo build -p qbz-external-reco` and `cargo build --workspace` (types
  used by `crates/qbz`'s `external_reco.rs` controller and the Slint state
  bridge for the Recommendations tab).
- `cargo test -p qbz-external-reco` (serde round-trip behavior for
  `AlbumReco`'s `default_source`/`subtitle` defaults is worth a quick manual
  check even without a dedicated test, since it's the trickiest coupling
  above).
- `cargo clippy -p qbz-external-reco`.
- Smoke-test importers: `grep -rn "external_reco::types::\|reco::types::"
  crates/qbz` — confirm every `ArtistReco`/`AlbumReco`/`TrackReco`/
  `ExternalCarousels` construction/field-access site in the qbz crate still
  compiles unchanged.
