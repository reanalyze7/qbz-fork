# crates/qbz-external-reco/src/lib.rs (174 lines)

## Summary
Crate root for the frontend-agnostic external-recommendations engine (Discover
"Recommendations" tab, ADR-006): declares the `RecoCatalog` trait the frontend
implements over its own `QbzCore`, the `RecoInputs`/handle structs, and a set of
thin async wrapper fns delegating to the `carousels` submodule (per-row builders
+ the convenience `build_external_carousels` combining them all).

## Proposed split
This file is only slightly over budget (174 vs 130) and is ALREADY mostly a
thin facade over `carousels`/`cache`/`types`/`validate`/`matching` submodules —
the split is really about separating "crate wiring" from "convenience
combinators", not extracting business logic (that already lives in the
submodules).

- `lib.rs` (~95 lines) — module doc, `pub mod cache; pub mod matching; pub mod
  types; mod carousels; mod validate;`, `use` imports, the `RecoCatalog` trait,
  `LastFmHandle`/`ListenBrainzHandle`/`RecoInputs` structs +
  `RecoInputs::has_external`, `is_cold_start`, `gather_history`, and `pub use`
  re-exports (`cache::RecoCache`, `carousels::{compose_artist_rails, ...}`,
  `types::{...}`). This keeps the trait + input types (the actual "public API
  shape" other crates implement against) as the crate root.
- `combine.rs` (~85 lines) — the per-row builder wrapper fns
  (`build_rec_artists_common`, `build_rec_artists_recent`, `build_rec_albums`,
  `build_fresh_releases`, `build_weekly_exploration`, `build_weekly_jams`,
  `build_deep_cut_albums`, `build_similar_albums_seeded`, `build_editorial`)
  plus the combining `build_external_carousels`. All are one-line-body
  delegations to `carousels::*` plus the final combinator's `tokio::join!` —
  moving them out of `lib.rs` leaves the trait/type definitions as the crate's
  conceptual "root".

## Re-export surface
`lib.rs` stays the crate's public surface: add `pub use combine::*;` (or
explicitly re-list each fn) so `qbz_external_reco::build_rec_artists_common`,
`qbz_external_reco::build_external_carousels`, etc. — called from whichever
frontend crate drives the Discover Recommendations tab — need zero changes.

## Coupling / watch out
- `is_cold_start`/`gather_history` (staying in `lib.rs`) are called FROM
  `combine.rs::build_external_carousels` — straightforward same-crate call,
  just confirm visibility (`pub(crate)` or `pub` is fine either way since both
  are in the same crate).
- `RecoInputs<'a>` carries several lifetime-bound references (`&'a
  MusicBrainzClient`, `&'a dyn RecoCatalog`, `Option<&'a Mutex<RecoCache>>`) —
  if `combine.rs`'s fns take `&RecoInputs<'_>` (they already do per the
  current signatures), no lifetime-annotation changes are needed on the move,
  just re-verify the elided lifetimes still compile once in a different file
  (should be identical, same generic signature).
- This crate is genuinely tiny/thin already — a two-file split is likely
  sufof enough; resist over-splitting into many sub-100-line files that would
  hurt readability more than help (the owner's rule is a ceiling, not a
  target).

## Verify after split
- `cargo build -p qbz-external-reco` and `cargo build --workspace` (whatever
  frontend crate implements `RecoCatalog` and calls `build_external_carousels`
  depends on this compiling cleanly).
- `cargo test -p qbz-external-reco` if any tests exist (check `carousels.rs`/
  `validate.rs`/`matching.rs` for `#[cfg(test)]` blocks not touched by this
  split).
- Smoke-test: `grep -rn "qbz_external_reco::" crates/` still resolves every
  call site (`RecoCatalog`, `RecoInputs`, `build_external_carousels`, the
  individual `build_*` fns, `RecoCache`, `compose_artist_rails`).
