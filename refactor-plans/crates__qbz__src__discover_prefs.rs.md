# crates/qbz/src/discover_prefs.rs (377 lines)

## Summary
Slint-side controller for Discover section configuration (Home/Editor's
Picks/For You tab ordering + enable/disable): owns the per-user
`DiscoverPrefsStore` lifecycle, pushes descriptor lists into `DiscoverState`,
and dispatches toggle/move/reset mutations from the configurator modal.

## Proposed split
By domain, following the file's own section-comment banners:

- `discover_prefs/mod.rs` (~50 lines) — module doc, `STORE`/`PREFS` statics,
  `init_for_user`, `teardown`, `current()`, `persist()`, `prefs_snapshot()`,
  re-exports of everything below.
- `discover_prefs/reco_ttl.rs` (~40 lines) — the Recommendations cache-TTL
  setting: `TTL_HOURS`, `ttl_index_from_hours`, `ttl_hours_from_index`,
  `set_reco_cache_ttl_index`, `reco_cache_ttl_secs`, plus
  `set_show_recommendations`.
- `discover_prefs/labels.rs` (~60 lines) — `render_kind()` and `label_for()`,
  the two big id-keyed match tables (pure lookup data, easiest to isolate).
- `discover_prefs/descriptors.rs` (~90 lines) — `bare_descriptor`,
  `foryou_descriptors`, `push_descriptors`, `push_config_rows`, `seed`.
- `discover_prefs/mutations.rs` (~70 lines) — `apply_after_mutation`,
  `on_open_configurator`, `on_close_configurator`, `on_toggle`, `on_move`,
  `on_reset`.

## Re-export surface
`discover_prefs/mod.rs` re-exports every `pub fn` currently called from
outside this module (`init_for_user`, `teardown`, `seed`,
`set_show_recommendations`, `set_reco_cache_ttl_index`,
`reco_cache_ttl_secs`, `on_open_configurator`, `on_close_configurator`,
`on_toggle`, `on_move`, `on_reset`, `prefs_snapshot`, `push_descriptors`,
`render_kind`, `label_for`) at `crate::discover_prefs::*` so `main.rs` and
`crate::home` (which calls `discover_prefs::prefs_snapshot` /
`push_descriptors`) need no changes.

## Coupling / watch out
- `PREFS`/`STORE` are process-global `Mutex<Option<_>>` statics — every
  submodule that mutates prefs (`reco_ttl.rs`, `mutations.rs`) needs access
  to them; keep them `pub(super)` (or `pub(crate)`) in `mod.rs` rather than
  re-declaring, to avoid two competing sources of truth.
- `apply_after_mutation` calls `crate::home::rerender_active_tab` and
  `crate::home::tab_descriptors` — a real cross-module coupling with the
  `home` module (outside this file); when actually splitting, keep those
  calls working via `crate::home::...`, not a relative import that would
  break once code moves into a subdirectory.
- `label_for` calls `qbz_i18n::mark(...)` at compile-registration time and
  `push_config_rows` calls `qbz_i18n::t(...)` at read time — these must stay
  paired (mark in `labels.rs`, the `t()` lookup can stay in
  `descriptors.rs`); don't accidentally use `mark` output directly as a
  runtime string.
- `seed()` touches THREE different Slint globals (`SettingsState`,
  `ExternalRecoState`, plus reading `crate::ui_prefs::load()`) — a fairly
  wide fan-out; keep it together with `push_descriptors` in
  `descriptors.rs` rather than splitting further.

## Verify after split
- `cargo check -p qbz` (this crate is the Slint frontend binary/lib).
- Manual smoke-test: open Settings > Discover configurator, toggle/move/
  reset a section on each of the three tabs, confirm persistence survives
  an app restart (reads `discover_prefs.json`-equivalent store).
