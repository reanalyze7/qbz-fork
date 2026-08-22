# crates/qbz-ui/ui/settings/SettingsExportModal.slint (192 lines)

## Summary
Single small modal: scrim, centered card, header (title + close-X),
explanation text, one "include auth" checkbox row + warning line, footer
(Cancel / Export). All state lives in the `SettingsExportState`/
`SettingsExportActions` globals — the modal is pure presentation.

## Proposed split
- `SettingsExportModal.slint` (~90 lines) — **stays the public re-export/
  root**: scrim + card chrome + header + composes the two blocks below.
- `SettingsExportAuthToggle.slint` (~55 lines, new) — the include-auth
  checkbox row + its indented warning line (lines 88-141); reads/writes
  `SettingsExportState.include-auth` directly (no props needed, it's a
  global).
- `SettingsExportFooter.slint` (~50 lines, new) — Cancel/Export button
  row (lines 143-188); calls `SettingsExportState.open = false` and
  `SettingsExportActions.confirm()` directly.

Given both extracted pieces are self-contained (drive the same global
state, no props to thread), this is a low-risk mechanical split — mostly
cut/paste plus adding `import { SettingsExportState, SettingsExportActions
} from "../state.slint";` to the new files.

## Re-export surface
`SettingsExportModal.slint` keeps exporting `SettingsExportModal`; no
property/callback surface to preserve since the modal only reads globals
(`visible: SettingsExportState.open`).

## Coupling / watch out
- Everything is driven by globals (`SettingsExportState`,
  `SettingsExportActions`), so there is no prop-threading risk — the only
  thing to get right is importing the globals in each new file.
- Rust side (`crate::settings::export_settings`) reads `include-auth` and
  drives the actual save-dialog/bundle-write flow — untouched by this
  purely presentational split.

## Verify after split
- `cargo build -p qbz-ui`.
- Open Settings > Developer > Export settings, toggle include-auth,
  confirm export still triggers the native save dialog.
