# crates/qbz/src/pinned.rs (210 lines)

## Summary
Per-user pinned-items lifecycle + thin accessor wrapper over the headless
`qbz_app::settings::pinned_items::PinnedItemsService`: a process-global
`Mutex<Option<Service>>` bound per session (init_for_user/teardown), fail-open
read accessors (is_pinned/list/count/keys_snapshot), fail-with-error mutations
(pin/unpin), plus one combined lifecycle round-trip test (~80 lines).

## Proposed split
This file is only marginally over budget (210 lines) — a light split by
pure-boundary (lifecycle vs accessors vs mutations vs tests) is enough; no
need for deep sub-domain splitting like the larger files in this batch.

- `pinned/mod.rs` (~30 lines) — module doc, imports, the `SERVICE` static, the
  `NO_SESSION_ERR` const, `pub use` re-export of `PinnedItem` from
  `qbz_app::settings::pinned_items`, and `pub use` re-exports of the split
  files' fns so `crate::pinned::is_pinned` etc. keep working.
- `pinned/lifecycle.rs` (~25 lines) — `init_for_user`, `teardown`.
- `pinned/read.rs` (~35 lines) — `with_service` helper, `is_pinned`, `list`,
  `count`, `keys_snapshot`.
- `pinned/write.rs` (~25 lines) — `mutate` helper, `pin`, `unpin`.
- `pinned/tests.rs` (~85 lines) — the `#[cfg(test)] mod tests` block
  (`unique_temp_dir`, `item`, `lifecycle_roundtrip`).

Given the file is only ~80 lines over budget, an alternative lighter-touch
split is just TWO files: `pinned/mod.rs` (lifecycle + accessors + mutations,
~130 lines) and `pinned/tests.rs` (~85 lines) — pick whichever the other
agents' emerging convention favors (check a few already-written plans for
crates/qbz's other small-overage files) for consistency across the crate.

## Re-export surface
`pinned/mod.rs` is the public surface: `pub use lifecycle::{init_for_user,
teardown}; pub use read::{is_pinned, list, count, keys_snapshot}; pub use
write::{pin, unpin}; pub use qbz_app::settings::pinned_items::PinnedItem;`.
Every call site (`crate::pinned::is_pinned(...)`, used from favorites.rs'
`map_artist` FavoriteArtistItem construction and elsewhere) needs zero changes.

## Coupling / watch out
- `SERVICE` (the `Mutex<Option<PinnedItemsService>>` static) is read by BOTH
  `read.rs::with_service` and written by `lifecycle.rs::init_for_user`/
  `teardown` and consulted by `write.rs::mutate` — three files touching one
  static; keep it defined once in `mod.rs` as `pub(super)` (or move to
  whichever file is considered canonical, e.g. `lifecycle.rs`, and have the
  others `use super::SERVICE`).
- `NO_SESSION_ERR` const is used in `write.rs::mutate` only — could move
  entirely into `write.rs` rather than staying in `mod.rs`, simplifying the
  cross-file surface.
- The doc comment explains this module deliberately mirrors
  `artist_blacklist`/`fav_cache`/`discover_prefs` sibling per-user stores in
  lifecycle shape (process-global `Mutex<Option<Service>>`,
  `init_for_user`/`teardown`, fail-open reads, fail-with-string-error
  mutations) — when splitting, keep a comment cross-referencing those sibling
  modules so a reader of just `lifecycle.rs` still sees the family pattern
  note, not just the top-level module doc.
- Tests use the SAME process-global `SERVICE` static and are explicitly
  written as ONE combined test (not parallel tests) specifically because the
  singleton is process-global and parallel tests would clobber each other —
  keep this constraint documented in `tests.rs` even after the split (don't
  let a future contributor "helpfully" split `lifecycle_roundtrip` into
  several `#[test]` fns).

## Verify after split
- `cargo build -p qbz`.
- `cargo test -p qbz pinned` — `lifecycle_roundtrip` green.
- Smoke-test: `grep -rn "pinned::" crates/qbz/src` still resolves (check
  `pinned::is_pinned`, `pinned::init_for_user`, `pinned::teardown`,
  `pinned::pin`, `pinned::unpin`).
