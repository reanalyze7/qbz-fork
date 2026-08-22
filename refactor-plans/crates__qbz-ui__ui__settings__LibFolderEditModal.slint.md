# crates/qbz-ui/ui/settings/LibFolderEditModal.slint (402 lines)

## Summary
Local Library folder-settings modal: alias, enabled toggle, network-share
override + fs-type select, accessibility status, change-path, last-
scanned, scan-this-folder — all driven by `LibFolderEditState` /
`LibraryManageActions`. One private `Field` labeled-section helper plus
the large modal body.

## Proposed split
- `LibFolderEditModal.slint` (~90 lines) — **stays the public re-export/
  root**: scrim, card chrome, `fs-type-string()` helper function, title +
  close-X, composes the sections below.
- `LibFolderFieldHelper.slint` (~15 lines, new) — the `Field` component
  (lines 21-31), imported by both this file and the sections below.
- `LibFolderLocationField.slint` (~90 lines, new) — the "Folder location"
  block: icon + path text + Change button + accessibility status (lines
  100-188).
- `LibFolderToggles.slint` (~110 lines, new) — Display-name `Field` +
  Enabled toggle + Network-override toggle + conditional FS-type
  `QbzSelect` (lines 190-283).
- `LibFolderFooter.slint` (~100 lines, new) — Last-scanned row + divider +
  footer (Scan/Cancel/Save buttons) (lines 285-398); the Save button calls
  `LibraryManageActions.save-folder-settings(...)` using the parent's
  `fs-type-string()` helper — either pass the already-stringified fs-type
  as a property, or duplicate/move the tiny pure function into this file.

## Re-export surface
`LibFolderEditModal.slint` keeps exporting `LibFolderEditModal`; no
external props/callbacks to preserve (fully global-state-driven via
`visible: LibFolderEditState.open`).

## Coupling / watch out
- `fs-type-string(i: int) -> string` (lines 38-48) is called only from the
  Save button in the footer — if the footer moves to its own file, either
  keep the function on `root` and call `root.fs-type-string(...)` from
  the extracted footer, or move the function itself into
  `LibFolderFooter.slint` (simpler, since it's the only caller).
- The `#619` hotkey-guard workaround on the alias `LineEdit`
  (`guard-focused` local property mirrored into
  `UiFocusState.text-input-focused`) is a documented gotcha — preserve the
  comment and pattern verbatim if that field moves file.
- Save button reads 6 different `LibFolderEditState` fields directly plus
  the derived fs-type string — keep the call's argument order/types exact.

## Verify after split
- `cargo build -p qbz-ui`.
- Manually open Folder settings from Settings > Library, toggle
  network/enabled, change fs-type, change path, scan, save, and cancel.
