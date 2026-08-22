# crates/qbzd/src/tui/screens/audio.rs (1094 lines)

## Summary
The daemon TUI's Audio settings screen (ratatui): staged-form model
(`StagedAudio`), a declarative field constraint matrix (`row_state`) + cross-setting
cascade rules re-derived 1:1 from the desktop `qbz` crate's ALSA device-grouping
logic, the `AudioState` screen controller (focus nav, popup editors, save-diffing),
and its render path — plus ~230 lines of unit tests at the bottom.

## Proposed split
This is the largest of my assigned files after `main.rs`; split by the file's own
section banners (`// ==== ... ====`), which already separate pure logic from
screen-state/render:

- `audio/mod.rs` (~40 lines) — module doc, imports, `pub use` re-exports of
  `StagedAudio`, `AField`, `AudioState`, `DeviceEntry`, `backend_label` (the public
  surface other TUI code touches — e.g. the App shell calling into this screen).
- `audio/model.rs` (~65 lines) — `StagedAudio` struct + `from_settings` (pure data
  mapping from `AudioSettings`).
- `audio/fields.rs` (~65 lines) — `AField` enum, `row_state()` (constraint matrix),
  `visible_fields()` — the declarative "what's shown/enabled" logic, pure and
  already unit-tested independently.
- `audio/cascades.rs` (~40 lines) — `cascade_on_toggle()`, `cascade_on_backend_change()`
  — the cross-setting cascade rules (§3.2.3 items 1-7), pure.
- `audio/device_grouping.rs` (~110 lines) — `AlsaSection` enum, `alsa_section()`,
  `alsa_section_label()`, `device_is_bit_perfect()`, `DeviceEntry` struct,
  `group_devices()` — the §3.2.2 picker-grouping logic, pure, ported 1:1 from
  desktop `settings.rs`.
- `audio/state.rs` (~230 lines) — `Editor` enum + `AudioState` struct fields +
  constructor + the non-render/non-input methods (`backend()`, `set_devices()`,
  `start_scan()`, `is_dirty()`, `is_editing()`, `editing_label()`,
  `focused_is_buffer()`, `save_keys()`, `mark_saved()`) — screen state management.
- `audio/input.rs` (~250 lines) — `impl AudioState` continued: `handle_key()`,
  `activate()`, `open_backend_picker()`, `open_device_picker()`,
  `open_alsa_plugin_picker()`, `open_dsd_picker()`, `handle_editor_key()` — all
  keyboard/popup-interaction logic (a second `impl AudioState` block in this file).
- `audio/render.rs` (~130 lines) — `impl AudioState` continued: `draw()`,
  `group_block()`, `field_block()`, `field_display()`, `device_label()` — the ratatui
  drawing path (a third `impl AudioState` block).
- `audio/labels.rs` (~55 lines) — `backend_label()`, `backend_value()`,
  `alsa_plugin_label()`, `alsa_plugin_value()`, `dsd_label()`, `short_device()` —
  value<->label mapping helpers, pure.
- `audio/tests.rs` (~230 lines) — the entire `#[cfg(test)] mod tests` block,
  unchanged, referencing everything via `use super::*`.

## Re-export surface
`audio/mod.rs` re-exports the public items so `crate::tui::screens::audio::{AudioState,
StagedAudio, AField, DeviceEntry, backend_label}` (whatever the App shell in
`crate::tui::app` imports) keeps compiling unchanged. Internally, `state.rs`,
`input.rs`, and `render.rs` all impl the same `AudioState` type across three files —
Rust supports multiple `impl` blocks for one struct in the same crate, so this is safe
as long as all three `use super::{AudioState, Editor, ...}` correctly and the struct
+ its private fields (`baseline`, `staged`, `focus`, `devices`, `scanning`, `editor`)
are defined once in `state.rs` and visible (via `pub(super)` or same-module privacy)
to `input.rs`/`render.rs`.

## Coupling / watch out
- **Private-field access across impl-block files**: `AudioState`'s fields are
  currently private (module-private) and accessed directly by `handle_key`,
  `draw`, etc. When these move to sibling files under `audio/`, Rust's privacy model
  requires the struct fields to be visible to those sibling modules too — since they
  are all still submodules of `audio/`, plain (non-`pub`) fields declared in
  `state.rs` remain visible to `input.rs`/`render.rs` AS LONG AS they're all
  children of the same `audio` module (Rust privacy is based on module tree
  ancestry, not file location) — should just work, but verify with `cargo check`
  immediately after moving, this is the single trickiest part of this split.
- `Editor` enum (with its `SelectPopup` variants and the `DsdConfirm` guard variant)
  is used by both `state.rs` (field decl) and `input.rs` (all the popup-resolution
  match arms) — keep `Editor` defined in `state.rs` (or promote to `mod.rs`) so both
  see it.
- `row_state()` is called from `fields.rs` (defines it), but also from `state.rs`
  (`focused_is_buffer`), `input.rs` (`activate`), and `render.rs` (`field_block`) —
  three cross-file call sites into `fields.rs`, all straightforward `use` imports.
- `group_devices()` (device_grouping.rs) is called from `state.rs`'s `set_devices()`
  — cross-file dependency, straightforward.
- The extensive `§3.2.x` spec-section doc comments must travel with their function.
- Test module references dozens of items across all the new files (`StagedAudio`,
  `AField`, `row_state`, cascades, `group_devices`, `AlsaSection`, `alsa_section`,
  `AudioState`, `AudioSettings`) — after the split, `tests.rs`'s `use super::*` from
  `mod.rs` needs `mod.rs` to actually re-export ALL of these (not just the
  intentionally-public API), so either keep `tests.rs` doing `use super::model::*;
  use super::fields::*; ...` explicitly per submodule, or make `mod.rs` glob
  re-export everything with `pub(crate) use`.

## Verify after split
- `cargo test -p qbzd tui::screens::audio` — all ~20 existing tests (cascades,
  constraint matrix, device grouping, save-diffing) must stay green.
- `cargo check -p qbzd` for the App shell's usage of this screen.
- Manual/TUI smoke-test if feasible: run the daemon TUI, open Audio screen, toggle a
  few fields, confirm cascades still fire and save-diff still only emits changed keys.
