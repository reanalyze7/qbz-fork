# crates/qbz/src/myqbz_prefs.rs (257 lines)

## Summary
Per-user persistence for the "My QBZ" sidebar branding (custom label + custom
icon path), stored as `<data_dir>/qbz/users/<user_id>/myqbz_branding.json`:
read/write helpers, label-coercion logic, and Slint-facing `seed`/`pick_icon`
functions that bridge the store to the `MyQbzBrandingState` global.

## Proposed split
By pure-data vs I/O vs Slint-bridging — a clean pure/IO/render-ish split
despite this being backend Rust, not UI code.

- `myqbz_prefs/mod.rs` (~25 lines) — module declarations + re-exports of
  `DEFAULT_LABEL`, `init_for_user`, `set_label`, `set_icon_path`,
  `reset_icon`, `seed`, `reseed_weak`, `pick_icon`.
- `myqbz_prefs/store.rs` (~90 lines) — `Branding` struct (+ `Default`,
  `default_label`), `USER_ID` static, `store_path`, `read`, `write`,
  `init_for_user`. The pure-JSON-file persistence layer.
- `myqbz_prefs/actions.rs` (~55 lines) — `coerce_label`, `set_label`,
  `set_icon_path`, `reset_icon`. The small business-logic layer that sits on
  top of `store.rs`'s `read`/`write`.
- `myqbz_prefs/ui_bridge.rs` (~70 lines) — `resolve`, `seed`, `reseed_weak`,
  `pick_icon`. The Slint-facing glue: decoding the custom icon image,
  pushing to `MyQbzBrandingState`, and the async native file-picker flow.
- `myqbz_prefs/tests.rs` (~35 lines) — the existing `#[cfg(test)] mod tests`
  block (`coerce_blank_label_yields_default`, `branding_defaults`,
  `legacy_json_without_fields_deserializes`,
  `missing_icon_path_field_keeps_label`), included via
  `#[cfg(test)] mod tests;` from `mod.rs`.

## Re-export surface
`myqbz_prefs/mod.rs` is the public-API surface — re-export `DEFAULT_LABEL`,
`init_for_user`, `set_label`, `set_icon_path`, `reset_icon`, `seed`,
`reseed_weak`, `pick_icon` at the same path so
`crate::myqbz_prefs::{init_for_user, seed, pick_icon, ...}` call sites in
the shell-entry / Settings-Appearance UI-glue code are unaffected.

## Coupling / watch out
- `store.rs`'s `Branding` struct and `read`/`write`/`store_path` functions
  must be `pub(super)` (not private) so `actions.rs` and `ui_bridge.rs`
  (specifically `resolve()`, which calls `read()`) can use them across the
  new file boundary.
- The `USER_ID: LazyLock<Mutex<Option<u64>>>` static is process-global mutable
  state — it's the one piece of real shared state in this file; keep it
  defined exactly once in `store.rs` and don't duplicate it if any other
  file seems to need direct access (route everything through `store_path()`
  instead).
- `resolve()` (in `ui_bridge.rs`) calls `read()` (in `store.rs`) and does
  `slint::Image::load_from_path` — this is the one place where store I/O and
  Slint-image I/O mix; keep it together in `ui_bridge.rs` rather than trying
  to force a strict pure/IO boundary here, since the doc comment explains a
  load failure must NOT mutate the store (that invariant spans both
  concerns).
- `pick_icon` spawns onto a `tokio::runtime::Handle` and calls
  `reseed_weak` (which does `weak.upgrade_in_event_loop`) — this
  thread-hopping (background async task -> UI event loop) is delicate;
  don't reorder the `set_icon_path` / `reseed_weak` calls when moving.

## Verify after split
- `cargo build -p qbz`
- `cargo test -p qbz myqbz_prefs` (all 4 existing tests green, unchanged
  assertions)
- Grep for `myqbz_prefs::` call sites (shell entry, Settings > Appearance,
  sidebar branding row) to confirm no import path broke.
- Manual smoke test: change the My QBZ label and custom icon in Settings,
  confirm the sidebar row updates live and the choice persists across a
  restart.
