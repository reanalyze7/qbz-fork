# crates/qbz-app/src/settings/developer.rs (213 lines)

## Summary
Developer-mode settings persistence: a `DeveloperSettings` data struct (just
`force_dmabuf` today), a SQLite-backed `DeveloperSettingsStore` (readonly-open,
normal-open, get/set), and a thread-safe `DeveloperSettingsState` host-state
wrapper — plus a `#[cfg(test)] mod tests` block (4 tests).

## Proposed split
Small file, only marginally over budget. Split by pure-data vs
store(IO)/state vs tests:

- `developer/mod.rs` (~40 lines) — module doc, `DeveloperSettings` struct +
  its `Default` impl (the one pure-data piece), and `pub use` of
  `DeveloperSettingsStore`/`DeveloperSettingsState` from the submodule
  below.
- `developer/store.rs` (~100 lines) — `DeveloperSettingsStore` struct +
  its full impl (`new_readonly`, `new_readonly_at_path`, `open_at`
  (private), `new`, `new_at`, `get_settings`, `set_force_dmabuf`) — this is
  the "IO" half (SQLite open/schema/query).
- `developer/state.rs` (~25 lines) — `DeveloperSettingsState` struct + its
  impl (`new`, `new_empty`) + its `Default` impl — the thread-safe host-
  state wrapper (`Arc<Mutex<Option<DeveloperSettingsStore>>>`).
- `developer/tests.rs` (~70 lines) — the existing `#[cfg(test)] mod tests`
  block (`unique_test_dir`, `fresh_store` helpers +
  `developer_settings_default_values_are_stable`,
  `developer_settings_store_returns_defaults`,
  `developer_settings_persist_force_dmabuf`,
  `developer_settings_readonly_opens_existing_db`), moved verbatim.

## Re-export surface
`developer/mod.rs` is the target of the existing `mod developer;` (or `pub
mod developer;`) declaration in `crates/qbz-app/src/settings/mod.rs`.
`DeveloperSettings`, `DeveloperSettingsStore`, `DeveloperSettingsState` must
all remain reachable as `crate::settings::developer::{...}` (or however
`qbz-app`'s settings module re-exports them upward, e.g.
`qbz_app::settings::developer::DeveloperSettingsState` used by the Tauri
command wrappers per the module doc's note that "Tauri command wrappers...
stay outside `qbz-app`") via `pub use store::DeveloperSettingsStore; pub use
state::DeveloperSettingsState;` in `mod.rs`.

## Coupling / watch out
- `DeveloperSettingsStore::new()` and `::new_at()` both funnel into the
  private `open_at()` helper (schema creation + WAL pragma) — keep `new`,
  `new_at`, `new_readonly`, `new_readonly_at_path`, and `open_at` together
  in `store.rs` since they share the exact db filename constant
  (`"developer_settings.db"`) inline in two call sites — consider hoisting
  that literal into a `const DB_FILE: &str = "developer_settings.db";` in
  `store.rs` while doing the actual split, to avoid the string existing
  twice.
- `DeveloperSettingsState::new()` calls `DeveloperSettingsStore::new()`
  directly — `state.rs` needs `use super::store::DeveloperSettingsStore;`
  (or `use crate::settings::developer::store::DeveloperSettingsStore;`).
- The doc comment "Tauri command wrappers and restart messaging stay
  outside `qbz-app`" signals this module is intentionally kept portable/
  UI-framework-agnostic — preserve that framing in `mod.rs`'s doc comment
  so the split doesn't accidentally invite Tauri-specific code creeping
  into `store.rs`/`state.rs`.

## Verify after split
- `cargo test -p qbz-app` — all 4 existing tests must pass unchanged
  (they use temp dirs via `std::env::temp_dir()` + a nonce, so they're
  already isolated and safe to move verbatim).
- `cargo check -p qbz-app` and check whatever Tauri-side command wrapper
  crate calls `DeveloperSettingsState`/`DeveloperSettingsStore` still
  compiles against the new module layout.
- Manual smoke-test: toggle the "Force DMA-BUF" developer setting in the
  running app's Settings > Developer panel, restart, confirm it persisted.
