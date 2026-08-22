# crates/qbz-app/src/settings/graphics.rs (339 lines)

## Summary
Graphics settings persistence: the `GraphicsSettings` data struct (+
`Default`) and `GraphicsSettingsStore` (a SQLite-backed CRUD store with
readonly-open, migration-on-open, and per-field getters/setters), plus a
`#[cfg(test)]` module with 5 integration-style tests against real SQLite
files in a temp dir.

## Proposed split
By pure-data vs I/O vs tests (matches the project's pure/IO/render
convention directly, since this file already cleanly separates the plain
struct from the SQLite store):

- `graphics/mod.rs` (~10 lines) — module doc (adapted from lines 1-5) +
  `pub use settings::GraphicsSettings; pub use store::GraphicsSettingsStore;`.
- `graphics/settings.rs` (~35 lines) — lines 11-45: `GraphicsSettings`
  struct + its `Default` impl. Pure data, no I/O.
- `graphics/store.rs` (~130 lines) — lines 47-219: `GraphicsSettingsStore`
  struct + impl (`new_readonly`, `new_readonly_at_path`, `open_at`, `new`,
  `new_at`, `get_settings`, and the six `set_*` setters). This is right at
  budget; if it lands over, split the six near-identical `set_*` setters
  (lines 150-218, ~70 lines) into `graphics/store/setters.rs` as a second
  `impl GraphicsSettingsStore` block, keeping `open_at`/`new*`/`get_settings`
  in `graphics/store.rs`.
- `graphics/tests.rs` (~120 lines) — lines 221-338: the entire
  `#[cfg(test)] mod tests` (unique_test_dir helper + 5 tests). Keep as
  `#[cfg(test)] mod tests;` declared from `graphics/mod.rs` (or from
  `store.rs` if the project convention is per-file tests rather than a
  sibling tests module — check a sibling already-split file in
  `refactor-plans/` for the house style, e.g.
  `crates__qbz-app__src__diagnostics.rs.md` keeps tests un-split since that
  file had none; here there IS a substantial test module, so a dedicated
  `graphics/tests.rs` referenced via `#[path = "tests.rs"] mod tests;` or a
  plain `mod tests;` inside `graphics/store.rs` both work — prefer keeping
  the test module physically next to `store.rs` since all 5 tests exercise
  `GraphicsSettingsStore`).

## Re-export surface
`graphics/mod.rs` (replacing the current single `graphics.rs`) re-exports
`GraphicsSettings` and `GraphicsSettingsStore` at `qbz_app::settings::
graphics::{GraphicsSettings, GraphicsSettingsStore}` exactly as today — the
parent `settings/mod.rs`'s existing `pub mod graphics;` (or equivalent)
declaration does not need to change at all, since Rust resolves
`mod graphics;` to either `graphics.rs` or `graphics/mod.rs` transparently.

## Coupling / watch out
- `GraphicsSettingsStore::open_at` is the single shared migration path used
  by both `new()` and `new_at()` — keep it un-split; it does sequential
  `ALTER TABLE ADD COLUMN` migrations (lines 101-112) that must stay in
  this exact order (each is a separate `execute_batch` whose error is
  deliberately swallowed via `let _ =`, since SQLite errors if the column
  already exists — this is the de facto migration mechanism, don't
  "clean up" the ignored errors).
  the ignored errors during a split — that's how repeated `open_at` calls
  on an existing DB stay idempotent).
- `new_readonly_at_path` is called directly by the test
  `graphics_settings_readonly_opens_existing_db` — if `store.rs` splits
  further into `store.rs` + `store/setters.rs`, `new_readonly_at_path` must
  stay `pub` and reachable via the same `GraphicsSettingsStore::` path.
- Doc comment on the crate/module (lines 1-5) states scope boundaries
  ("Startup detection, environment variable application, crash recovery,
  and command transport stay outside `qbz-app`") — preserve this doc
  comment verbatim on `graphics/mod.rs`, it's a deliberate architectural
  note for future maintainers, not filler.
- Every setter method (`set_hardware_acceleration`, `set_force_x11`, etc.)
  follows an identical `self.conn.execute(...).map_err(...)` pattern — no
  shared helper currently extracts this; fine to leave as-is (don't
  introduce a generic macro/helper as part of a pure line-count split,
  that's a separate refactor with its own risk).

## Verify after split
- `cargo check -p qbz-app` and `cargo test -p qbz-app graphics` (or
  `cargo test -p qbz-app -- graphics::` depending on test path) to confirm
  all 5 tests still pass — they create real temp-dir SQLite files, so also
  confirm no leftover temp dirs/handles from a botched split (each test
  does `std::fs::remove_dir_all(dir)` cleanup already).
- Smoke-test the Settings > Graphics screen in the running desktop app
  (`qbz-app`/`qbz` bin) to confirm a live read/write of hardware
  acceleration or preferred GPU still persists across a restart.
