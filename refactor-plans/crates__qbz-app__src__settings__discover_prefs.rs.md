# `crates/qbz-app/src/settings/discover_prefs.rs` (717 lines)

## 1. Summary
Frontend-agnostic port of the Tauri `discovery-v2/sectionPrefs.ts` store: the
per-tab (Home/EditorPicks/ForYou), ordered, enable/disable section
preferences model (pure logic: defaults, toggle, move, reset, migrate,
reconcile) plus a thin SQLite-backed `DiscoverPrefsStore` wrapper and its
`#[cfg(test)]` test suite (~215 lines of tests).

## 2. Proposed module layout

Convert to `discover_prefs/` directory:

- `discover_prefs/mod.rs` (~35) — crate-level doc comment (moved here
  verbatim), `mod` declarations, `pub use` re-exports of every public item
  (`DiscoveryTab`, `DiscoverySectionId`, `SectionPref`, `DiscoverPrefs`,
  `default_prefs`, `reconcile_list`, `DiscoverPrefsStore`,
  `DiscoverPrefsState`, `create_empty_discover_prefs_state`). **This is the
  re-export/public-API surface.**
- `discover_prefs/tabs.rs` (~30) — `DiscoveryTab` enum + `as_key`/`from_key`/
  `ALL`.
- `discover_prefs/section_id.rs` (~90) — `DiscoverySectionId` enum +
  `as_str`/`from_str`.
- `discover_prefs/model.rs` (~120) — `SectionPref` struct, `pref()` helper,
  `DiscoverPrefs` struct definition, and `default_prefs()` (the big literal
  default lists — keep these together since they're the "spec" and are
  easiest to review as one unit; if it creeps over 130 once moved, split
  `default_prefs()` itself into `default_prefs.rs` separate from the struct
  definition).
- `discover_prefs/ops.rs` (~95) — `impl DiscoverPrefs` behavioral methods:
  `tab`, `tab_mut`, `toggle`, `move_section`, `reset_tab`, `is_enabled`,
  `enabled_count`, `enabled_ordered`, `available_ids`.
- `discover_prefs/json.rs` (~110) — `impl DiscoverPrefs::to_json`,
  `DiscoverPrefs::migrate`, and the free function `reconcile_list`.
- `discover_prefs/store.rs` (~90) — `DiscoverPrefsStore` struct + `open_at`/
  `new`/`new_at`/`load`/`save`, plus `DiscoverPrefsState` type alias and
  `create_empty_discover_prefs_state()`.
- Tests split into `discover_prefs/tests/` (or a single
  `discover_prefs/tests.rs` referenced via `#[cfg(test)] mod tests;` from
  `mod.rs`, itself split by `mod` if it exceeds 130):
  - `tests/defaults_and_ops.rs` (~100) — "Group 1" defaults spec, "Group 4"
    move_section, "Group 5" toggle/reset_tab tests.
  - `tests/reconcile_and_migrate.rs` (~80) — "Group 2" reconcile_list,
    "Group 3" migrate tests.
  - `tests/store_roundtrip.rs` (~55) — "Group 6" store round-trip +
    corruption recovery (needs the `unique_test_dir` helper).

## 3. Re-export / public API surface
`discover_prefs/mod.rs` is what other modules already import via
`crate::settings::discover_prefs::{...}` (e.g. wherever Discover UI wiring
in `qbz-app` reads prefs). Since `discover_prefs.rs` currently has no
`mod.rs`-style re-export layer (it's a flat file), converting it to a
directory with `mod.rs` re-exporting everything under the same names keeps
every existing `use crate::settings::discover_prefs::DiscoverPrefs;` etc.
working unchanged.

## 4. Tricky coupling to watch
- `DiscoverPrefs::migrate` and `reconcile_list` are tightly coupled (migrate
  calls reconcile_list three times) — keep them in the same file
  (`json.rs`) rather than splitting further, since they share the exact
  "persisted vs. fallback" contract described in the doc comments.
- `default_prefs()` is called from `DiscoverPrefsStore::open_at` (to seed a
  fresh DB row) AND from test code AND from `DiscoverPrefs::migrate`'s
  fallback branches — it must stay `pub fn` at the `discover_prefs::` path
  (not `pub(crate)`), matching current visibility.
- The doc comment explicitly says the model is "PURE and headless-testable;
  the store is a thin wrapper" — preserve this pure/IO separation as the
  actual axis of the split (`model.rs`/`ops.rs`/`json.rs` = pure,
  `store.rs` = IO), which the section boundaries already suggested by the
  file's own `// ---- comments ----` largely mirror.
- Watch for any other module in `qbz-app` doing
  `use super::discover_prefs::reconcile_list` or similar relative imports
  that assume a flat file rather than a directory — a `grep -rn
  "discover_prefs::" crates/qbz-app/src` before/after the real split should
  catch these.

## 5. What to verify after the real split
- `cargo test -p qbz-app discover_prefs` — all 3 test groups above must
  stay green, including the SQLite round-trip + corruption-recovery test
  (exercises real file I/O in a temp dir).
- `cargo build -p qbz-app` to confirm all internal importers still resolve.
- Confirm `create_empty_discover_prefs_state()` / `DiscoverPrefsState` are
  still reachable from wherever the app wires up its settings state at
  startup (likely `qbz-app/src/settings/mod.rs` or `main.rs`).
