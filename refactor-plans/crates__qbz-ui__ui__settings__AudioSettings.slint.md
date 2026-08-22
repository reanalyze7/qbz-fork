# crates/qbz-ui/ui/settings/AudioSettings.slint (437 lines)

## Summary
The Audio settings panel: hearing-protection volume cap, streaming quality,
output device/backend routing, bit-perfect options (exclusive mode, DSD,
HiFi Wizard entry), and startup sync options — all bound to the
`SettingsState` global and emitting callbacks the Rust settings controller
handles.

## Proposed split
By settings group (each `GroupHeader` section is already a clean visual/
logical boundary), keeping the local `GroupHeader`/`Divider` helper
components in the top-level file since every section uses them.

- `AudioSettings.slint` (~90 lines) — becomes the orchestrator: imports +
  the two local helper components (`GroupHeader`, `Divider`), and the
  `AudioSettings` root component's `VerticalLayout` that mounts each
  section sub-component in order, wiring their callbacks through to its own
  `settings-bool`/`settings-select`/`settings-slider`/`settings-reset`/
  `settings-release-device` callbacks (pass-through).
- `settings/audio/HearingProtectionSection.slint` (~55 lines) — the
  "Protect your hearing" toggle + conditional volume-limit slider block
  (lines 42-92 in the current file). **Keep this one section intact as a
  single cohesive component** per the recent "Protect your hearing" addition
  — do not further subdivide the toggle from its slider.
  Exports `settings-bool(string, bool)` and `settings-slider(string, int)`.
- `settings/audio/StreamingSection.slint` (~55 lines) — streaming quality
  select, "limit quality to device" toggle, detected-device-limit readout,
  and the fallback disclosure text (lines 94-143).
  Exports `settings-bool`, `settings-select`.
- `settings/audio/OutputSection.slint` (~130 lines) — audio backend select,
  JACK warning banner, output device row (with refresh/release button +
  searchable select), ALSA plugin select, hardware volume toggle, DSD
  playback mode select (lines 149-264). This is the biggest section; if it
  lands over 130 lines after extraction, further split the output-device
  row (with its refresh button) into
  `settings/audio/OutputDeviceRow.slint`.
  Exports `settings-bool`, `settings-select`, `settings-release-device`.
- `settings/audio/BitPerfectSection.slint` (~110 lines) — exclusive mode,
  reserve DAC, DAC passthrough + force-bit-perfect, allow-quality-fallback,
  and the HiFi Wizard launch button (lines 270-374).
  Exports `settings-bool` and calls `DacWizardActions.open()` directly
  (no callback needed for that one, matching current behavior).
- `settings/audio/StartupSection.slint` (~35 lines) — sync-on-startup toggle
  and (PipeWire-only) lock-output-device toggle (lines 380-406).
  Exports `settings-bool`.
- `settings/audio/ResetRow.slint` (~30 lines) — the standalone reset button
  at the bottom (lines 408-436). Exports `settings-reset`.

## Re-export surface
`AudioSettings.slint` stays the single import surface other code uses
(`import { AudioSettings } from "settings/AudioSettings.slint";` is
unaffected — the root component's name and public callback signatures
(`settings-bool`, `settings-select`, `settings-slider`, `settings-reset`,
`settings-release-device`) are unchanged; it just delegates to the new
sub-components internally.

## Coupling / watch out
- All sections read from the single shared `SettingsState` global directly
  (not passed as props) — so each new sub-component can keep importing
  `SettingsState` from `../state.slint` independently; no prop-drilling
  needed, which keeps the split low-risk.
- Every toggle/select callback follows the identical pattern
  `{ SettingsState.X = v; root.settings-bool("X", v); }` — when moving a
  block, keep each callback's string key (`"exclusive-mode"`,
  `"dac-passthrough"`, etc.) byte-for-byte identical; these keys are almost
  certainly matched against string literals on the Rust side.
- `OutputSection`'s device row and `BitPerfectSection`'s rows both gate on
  `SettingsState.backend-is-alsa` / `backend-is-pipewire` (exclusive mode is
  ALSA-only, DAC passthrough is PipeWire-only, DSD mode is ALSA-only) — these
  are independent per-row conditions already read straight from the shared
  global, so splitting the sections into separate files introduces no new
  coupling.
- The local `GroupHeader`/`Divider` helper components (top of file) are used
  by every section; either duplicate the tiny definitions in each new file
  or (preferred) leave them in `AudioSettings.slint` and have Slint's import
  mechanism expose them to the sub-components via a shared
  `settings/audio/_common.slint` if Slint requires explicit imports between
  sibling files (verify Slint's import resolution before assuming implicit
  visibility).

## Verify after split
- Run the Slint viewer / `slint-viewer` compile check on `AudioSettings.slint`
  and each new sub-file.
- `cargo build -p qbz-ui` (or whichever crate embeds the `.slint` via
  `slint_build`) to confirm the generated bindings still expose
  `AudioSettings` with the same callback names.
- Manually open Settings > Audio in the running app and verify every row
  (especially the newly-added "Protect your hearing" section) still renders
  and its toggle/slider still round-trips through Rust.
