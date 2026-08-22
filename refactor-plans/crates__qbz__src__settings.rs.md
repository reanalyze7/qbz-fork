# crates/qbz/src/settings.rs (1529 lines)

## Summary
Slint settings controller: owns the Audio (`AudioSettingsStore`) and
Playback (`PlaybackPreferencesStore`) persistence stores plus the JSON
`ui_prefs` store, builds a `SettingsSnapshot` (device enumeration, ALSA
grouping, dropdown index maps), applies it to the `SettingsState` Slint
global, and handles every settings-panel callback (bool/slider/string/
select/reset/release-device/export) including cross-setting cascades and
live `Player` re-application.

## Proposed split
This is by far the largest file in the assigned set — split by domain into
a `settings/` module directory:

- `settings/mod.rs` (~40 lines) — module doc, `pub use` re-exports of every
  currently-public item (`SettingsCtx`, `SettingsSnapshot`, `load_snapshot`,
  `apply_snapshot`, `apply_startup_bitperfect_volume`, `refresh_device_cap`,
  `handle_bool`, `handle_slider`, `handle_string`, `handle_select`,
  `handle_reset`, `handle_release_device`, `export_settings`), plus the
  small shared `Apply` enum and `with_audio`/`with_playback` helpers (or put
  those in `settings/store.rs` — see below).
- `settings/store.rs` (~110 lines) — `SettingsCtx`, `SettingsMaps`,
  `with_audio`, `with_playback`, the `Apply` enum. This is the shared state
  every other submodule closes over.
- `settings/tables.rs` (~30 lines) — `DSD_MODES`, `ALSA_PLUGINS`,
  `RETRY_BEHAVIORS` const tables (i18n-marked label/value pairs).
- `settings/devices.rs` (~200 lines) — `DeviceList`, `DeviceRow`,
  `AlsaSection`, `alsa_section`, `alsa_section_label`,
  `device_is_bit_perfect`, `backend_label`, `enumerate_devices`,
  `group_alsa_devices`, `output_labels`. All the "what devices/backends are
  there and how do we label/group them" pure logic — this is the most
  self-contained chunk and the best pure-logic candidate (no store access,
  blocking I/O only via `BackendManager::create_backend`).
- `settings/snapshot.rs` (~180 lines) — `SettingsSnapshot` struct,
  `build_snapshot`, `load_snapshot`, `string_model`/`bool_model` helpers,
  `apply_snapshot`. Depends on `devices.rs` + `tables.rs` + `store.rs`.
- `settings/apply.rs` (~90 lines) — `apply_audio`,
  `maybe_force_bitperfect_volume`, `apply_startup_bitperfect_volume`,
  `refresh_device_cap`, `push_conditional_flags`, `rebuild_and_push`. The
  "push settings changes into the live Player + re-push UI" glue.
- `settings/handlers/mod.rs` (~10 lines) — re-exports the four handler
  submodules below.
- `settings/handlers/bool.rs` (~190 lines) — `handle_bool` (the largest single
  function: cascades, offline-mode routing, all bool keys) +
  `set_offline_mode`.
- `settings/handlers/slider.rs` (~35 lines) — `handle_slider` (buffer +
  crossfade seconds — **keep this whole file intact**, it's the recently-
  touched crossfade slider wiring).
- `settings/handlers/select.rs` (~170 lines) — `handle_select` (streaming
  quality, backend, device, dsd-mode, alsa-plugin, retry-behavior) +
  `handle_string` stub.
- `settings/handlers/reset.rs` (~40 lines) — `handle_reset`.
- `settings/export.rs` (~95 lines) — `export_settings` (the `.qbzb` bundle
  export flow — self-contained, no coupling to the rest).
- `settings/tests.rs` or keep `#[cfg(test)] mod tests` split alongside each
  submodule it tests (`devices.rs` gets the ALSA-section/backend-label
  tests, `tables.rs` gets the const-table tests). Prefer colocating tests
  with the code they exercise over one big test file.

## Re-export surface
`settings/mod.rs` becomes the `mod settings;` target. Every function/struct
currently reachable as `crate::settings::X` in `main.rs` and elsewhere must
stay reachable at that same path via `pub use store::*; pub use
snapshot::{SettingsSnapshot, load_snapshot, apply_snapshot}; pub use
apply::*; pub use handlers::*; pub use export::export_settings;`.

## Coupling / watch out
- `SettingsCtx` (in `store.rs`) is the shared state nearly every other
  submodule takes `&ctx`/`Arc<ctx>` — get its visibility (`pub(crate)`
  fields vs `pub` accessor) right first since everything else depends on it.
- `handle_bool`'s cross-setting cascades (dac-passthrough ↔ skip-sink-switch
  / pw-force-bitperfect; streaming-only ↔ gapless) are easy to break if
  split further — keep the whole cascade match block in one function/file.
- The task explicitly calls out that `settings.rs` was recently touched for
  the crossfade slider and offline-cache size-limit UI wiring — the
  crossfade branch lives in `handle_slider` (→ `settings/handlers/slider.rs`)
  and the offline-mode toggle lives in `handle_bool`/`set_offline_mode` (→
  `settings/handlers/bool.rs`). Keep each as one intact block during the
  split, don't interleave with unrelated cascade logic.
- `crate::session_persist::set_gates` is called from both `build_snapshot`
  and twice inside `handle_bool` ("persist-session"/"resume-position") —
  make sure both call sites land in modules that both import
  `crate::session_persist`.
- `ctx.maps` (`Mutex<SettingsMaps>`) is written in `build_snapshot`
  (`snapshot.rs`) and read in `handle_select`'s "backend"/"device" arms
  (`handlers/select.rs`) — a cross-module dependency on the same lock;
  don't let a stale copy diverge.
- `NowPlayingState` mirroring in `apply_snapshot` (output backend/mode
  labels) is a "the settings push also touches a different Slint global"
  detail — keep it inside `apply_snapshot` intact, don't split the two
  `SettingsState`/`NowPlayingState` writes apart.

## What to verify after the real split
- `cargo build -p qbz` compiles with no path changes needed in `main.rs`
  (grep `settings::` call sites first).
- `cargo test -p qbz` — all `#[cfg(test)]` tests (ALSA section
  classification, group_alsa_devices, alsa/retry table checks, backend
  label distinctness) still pass wherever they land.
- Manual/smoke: open Settings > Audio, flip a cascade toggle (e.g.
  dac-passthrough), confirm the dependent toggle flips too; move the
  crossfade slider; toggle offline mode; export settings via Settings >
  Developer.
