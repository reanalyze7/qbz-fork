# crates/qbz/src/reco_dismiss.rs (234 lines)

## Summary
Per-user "Not interested" dismissal store for Discover recommendations — a
small JSON-file-backed store (reco-scoped, not the app-wide blacklist),
bound/unbound per session.

## Proposed split
Split store-plumbing from the public API + tests:

- `reco_dismiss/mod.rs` (~65 lines) — `FILE_NAME`, `DismissedArtist`,
  `DismissStore`, `STORE_PATH` static, `init_for_user`, `teardown`,
  `pub use` of `io` and `ops`.
- `reco_dismiss/io.rs` (~40 lines) — `store_path`, `load_store`,
  `write_store` (the file-level read/write, fail-open on error).
- `reco_dismiss/ops.rs` (~55 lines) — `ids_snapshot`, `list`, `dismiss`,
  `remove` (the public read/mutate API).
- `reco_dismiss/tests.rs` (~77 lines) — existing `#[cfg(test)] mod tests`
  (one combined lifecycle test — keep as one test, per its own comment
  explaining the process-global path can't run in parallel tests).

## Re-export surface
`reco_dismiss/mod.rs` stays the `mod reco_dismiss;` target with
`DismissedArtist` defined there; `pub use ops::*;` keeps `ids_snapshot`,
`list`, `dismiss`, `remove` at `crate::reco_dismiss::X` (consumed by
`external_reco`'s paint filter and the Blacklist Manager's Recommendations
tab).

## Coupling / watch out
- `STORE_PATH: Mutex<Option<PathBuf>>` is shared between `mod.rs`
  (init_for_user/teardown) and `io.rs` (store_path) — keep it defined in
  `mod.rs`, `io.rs` does `use super::STORE_PATH;`.
- Fail-open semantics are load-bearing everywhere (no session, corrupt
  file, unbound mutation) — preserve every early-return branch exactly;
  this file has zero tolerance for a "helpful" error-propagation refactor.
- The `#[cfg(test)]` test intentionally stays a SINGLE function, documented
  as required because `STORE_PATH` is process-global — do not split it into
  parallel `#[test]` functions.

## Verify after split
- `cargo test -p qbz reco_dismiss::` — `lifecycle_roundtrip` green.
- `cargo build -p qbz` and confirm `external_reco`'s paint filter +
  Blacklist Manager's Recommendations tab still compile against
  `crate::reco_dismiss::X`.
