# crates/qbz-app/src/settings/tray.rs (416 lines)

## Summary
System-tray preferences: `TraySettings` struct (+ default +
`normalize_tray_icon_theme`), a SQLite-backed `TraySettingsStore`, and a
session-wrapper `TraySettingsState`. Has a `#[cfg(test)]` module covering
defaults, persistence, legacy-value normalization, and schema migration.

## Proposed split
- `tray/mod.rs` (~15 lines) — re-exports.
- `tray/prefs.rs` (~55 lines) — `TraySettings` struct, `Default` impl,
  `default_tray_icon_theme`, `normalize_tray_icon_theme` (lines 14-67).
- `tray/store.rs` (~150 lines) — `TraySettingsStore` struct + impl:
  `open_at` (schema + migration), `new`, `new_at`, `get_settings`,
  `set_enable_tray`, `set_minimize_to_tray`, `set_close_to_tray`,
  `set_tray_icon_theme`, `set_mac_hide_dock` (lines 68-219).
- `tray/state.rs` (~100 lines) — `TraySettingsState` struct + impl:
  `new`, `new_empty`, `init_at`, `teardown`, `get_settings`,
  `set_enable_tray`, `set_minimize_to_tray`, `set_close_to_tray`,
  `set_mac_hide_dock` (lines 220-312) — a thin passthrough wrapper over
  `TraySettingsStore` guarded by a lock/Option, same shape as
  `FavoritesPreferencesState`/`RemoteControlSettingsState` in sibling
  files.
- `tray/tests.rs` (~105 lines) — the `#[cfg(test)] mod tests` block
  (lines 313-416).

## Re-export surface
`tray/mod.rs` re-exports `TraySettings`, `normalize_tray_icon_theme`,
`TraySettingsStore`, `TraySettingsState` at their current
`qbz_app::settings::tray::X` paths. `crates/qbz/src/tray_settings.rs`
imports exactly these two: `qbz_app::settings::tray::{
normalize_tray_icon_theme, TraySettings}` and `TraySettingsState` — these
must not move.

## Coupling / watch out
- `crates/qbz/src/tray_settings.rs` is a confirmed external caller (`pub
  use qbz_app::settings::tray::{normalize_tray_icon_theme, TraySettings};
  use qbz_app::settings::tray::TraySettingsState;`) — keep those two
  exact paths working after the split.
- `open_at`'s schema migration logic (legacy value handling, per the
  test `tray_settings_migrates_legacy_schema`) must stay bundled with the
  store's `open_at`, not separated from the table-creation SQL.
- `normalize_tray_icon_theme` is a pure function reused by both the
  store's persistence path and (per `crates/qbz/src/tray_settings.rs`)
  external callers — keep it in `prefs.rs` alongside the struct it
  normalizes for, and make sure it's `pub`, not `pub(crate)`.

## Verify after split
- `cargo test -p qbz-app settings::tray` green (defaults, persistence,
  legacy-theme normalization, schema migration).
- `cargo build -p qbz` (the `tray_settings.rs` re-export) and `-p qbz-app`.
