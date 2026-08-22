# crates/qbz-reco/src/sparse_vector.rs (395 lines)

## Summary
`SparseVector` — parallel-array sparse vector (sorted indices + values) with
arithmetic (`add`/`sub`/`scale`/`dot`/`magnitude`/`normalize`/
`cosine_similarity`) used for artist-similarity math. Pure computation, no I/O.

## Proposed split
By operation category — this is the cleanest "pure computation" split in
the crate:

- `sparse_vector/mod.rs` (~70 lines) — struct def, `new`/`with_capacity`/
  `from_parts`/`set`/`get`/`remove`/`nnz`/`is_empty`/`indices`/`values`/`iter`
  (the basic accessors), `pub use` of `ops` module.
- `sparse_vector/ops.rs` (~95 lines) — `add`, `sub`, `scale`, `dot`,
  `magnitude`, `normalize`, `cosine_similarity` (all pure math, `impl
  SparseVector` block using `use super::SparseVector`).
- `sparse_vector/tests.rs` (~165 lines) — existing `#[cfg(test)] mod tests`
  (13 tests) — could split into `tests_basic.rs` (~55, set/get/remove) and
  `tests_ops.rs` (~110, add/dot/magnitude/normalize/cosine/scale) if the
  reviewer wants each test file under 130 too — both are already <130 as one
  file so a single `tests.rs` is fine.

## Re-export surface
`sparse_vector/mod.rs` stays the `mod sparse_vector;` target — `SparseVector`
struct + all its methods stay reachable at `crate::sparse_vector::SparseVector`
unchanged since `ops.rs` is just another `impl` block for the same type.

## Coupling / watch out
- Used heavily by `store.rs`, `builder.rs`, `suggestions.rs` — purely via the
  public struct/methods, no internal fields touched externally, so this
  split has zero blast radius outside the file.
- `Serialize`/`Deserialize` derive on the struct: field layout (`indices`,
  `values`) must stay identical for any persisted vectors.

## Verify after split
- `cargo test -p qbz-reco sparse_vector::` — all 13 tests green.
- `cargo check -p qbz-reco` for the three internal consumers.
