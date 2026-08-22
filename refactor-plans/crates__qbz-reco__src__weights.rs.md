# crates/qbz-reco/src/weights.rs (199 lines)

## Summary
`RelationshipWeights` config struct (weights per relationship type: band
membership, credits, Qobuz similarity, tags, user affinity) with presets
(`default`, `band_focused`, `similarity_focused`) and lookup helpers
(`weight_for_mb_relation`, `weight_for_source`).

## Proposed split
Small file, only ~69 lines over budget — split by struct-def vs behavior:

- `weights/mod.rs` (~55 lines) — `RelationshipWeights` struct definition +
  `pub use` of presets/lookup submodules.
- `weights/presets.rs` (~70 lines) — `impl Default for RelationshipWeights`,
  `band_focused()`, `similarity_focused()`.
- `weights/lookup.rs` (~50 lines) — `weight_for_mb_relation`,
  `weight_for_source`.
- `weights/tests.rs` (~35 lines) — existing `#[cfg(test)] mod tests`.

## Re-export surface
`weights/mod.rs` remains the `mod weights;` target; `RelationshipWeights` is
defined there so `crate::weights::RelationshipWeights` is unchanged. The
`impl` blocks in `presets.rs`/`lookup.rs` use `use super::RelationshipWeights;`.

## Coupling / watch out
- `weight_for_source` parses `"mb:"`-prefixed strings and calls back into
  `weight_for_mb_relation` — keep both in the same file (`lookup.rs`) to
  avoid an extra cross-file jump for a 3-line delegation.
- `RelationshipWeights` derives `Serialize`/`Deserialize` — field order/names
  must not change (used for persisted config, if any caller serializes it).

## Verify after split
- `cargo test -p qbz-reco weights::` — 3 existing tests green.
- `cargo check -p qbz-reco` — confirm `builder.rs` (`use crate::weights::RelationshipWeights`) still resolves.
