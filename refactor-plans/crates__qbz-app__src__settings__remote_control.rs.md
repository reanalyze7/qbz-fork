# crates/qbz-app/src/settings/remote_control.rs (729 lines)

## Summary
Two related but distinct domains sharing one file: (1) Remote Control
settings (`RemoteControlSettings`/`Store`/`State` — enabled/port/secure/
token, with token generation), and (2) the allowed-origins allowlist for
that remote-control server (`AllowedOrigin`/`Store`/`State`). Both follow
the same struct/Store/State three-tier pattern seen in `tray.rs`/
`favorites.rs`. Almost certainly the single largest natural split point in
this batch: split by domain first, then by tier within each.

## Proposed split
- `remote_control/mod.rs` (~15 lines) — re-exports both domains.
- `remote_control/settings.rs` (~190 lines) — `RemoteControlSettings`
  struct + `Default`, `RemoteControlSettingsStore` (open_at, new, new_at,
  get_settings, set_enabled, set_port, set_secure, set_token,
  regenerate_token), `generate_token()`, `ensure_secure_column()` (lines
  21-288).
- `remote_control/settings_state.rs` (~85 lines) — `RemoteControlSettingsState`
  struct + impl (lines 171-252, the thin session-wrapper tier — note this
  overlaps the settings.rs range above; the implementer should carve the
  exact boundary at line 171 where `RemoteControlSettingsState` begins).
- `remote_control/origins.rs` (~140 lines) — `AllowedOrigin` struct,
  `AllowedOriginsStore` (open_at, new, new_at, get_origins,
  is_origin_allowed, add_origin, remove_origin, restore_defaults) (lines
  289-427).
- `remote_control/origins_state.rs` (~90 lines) — `AllowedOriginsState`
  struct + impl (lines 428-518ish, verify exact end against the file's
  full 729 lines — this pass read through line 487; the implementer
  should confirm nothing after `AllowedOriginsState` is missed, e.g. a
  trailing `#[cfg(test)]` module).

## Re-export surface
`remote_control/mod.rs` must re-export `RemoteControlSettings`,
`RemoteControlSettingsStore`, `RemoteControlSettingsState`,
`AllowedOrigin`, `AllowedOriginsStore`, `AllowedOriginsState` at their
current `qbz_app::settings::remote_control::X` paths — check
`crate::api` or wherever the remote-control HTTP server / its
origin-check middleware lives for exact import paths before finalizing.

## Coupling / watch out
- `regenerate_token()`/`generate_token()` and the `set_token`/`set_secure`
  pair are security-sensitive (token used for remote-control auth) — keep
  them together in `settings.rs`, don't split token generation from its
  storage.
- `ensure_secure_column()` is a schema-migration helper — verify which
  struct's `open_at` calls it (likely `RemoteControlSettingsStore`) and
  keep it adjacent.
- `AllowedOriginsState.store` is `pub` (`pub store: Arc<Mutex<Option<...>>>`)
  unlike the other State wrappers in this batch which keep their guard
  private — if true, some caller reaches into the field directly; grep
  for `.store.lock()` on an `AllowedOriginsState` instance from outside
  this file before finalizing the split, since that's an unusual API
  surface to preserve exactly.
- Two independent "origins allowed" concerns (browser CORS-style allowlist
  vs remote-control enable/port/token) must not get tangled — they are
  read together by the HTTP server (is this port/token valid AND is this
  origin allowed) but persisted in separate stores/tables.

## Verify after split
- `cargo build -p qbz-app` and whichever crate hosts the remote-control
  HTTP server (likely `qbz` or `qbzd`).
- Manually exercise: enable remote control, regenerate token, add/remove
  an allowed origin, confirm the server still gates on both correctly.
