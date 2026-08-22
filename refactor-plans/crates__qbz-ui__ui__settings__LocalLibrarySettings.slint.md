# crates/qbz-ui/ui/settings/LocalLibrarySettings.slint (486 lines)

## Summary
Settings > Local Library panel: folder list management (add/remove/edit/
enable/scan) with a compact table, filter, scan-progress bar, an albums-view
grouping selector, maintenance (cleanup missing files), and a two-step danger
zone (clear library DB). Defines several small local components
(`GroupHeader`, `SecondaryButton`, `IconBtn`, `FolderTableHeader`,
`FolderRow`) used only within this file.

## Proposed split
- `LocalLibrarySettings.slint` (~110 lines) — becomes the top-level
  composition: imports, the `export component LocalLibrarySettings`
  skeleton (folders section header/toolbar/filter/table, scan progress,
  albums-view row, maintenance row, danger-zone row), delegating the local
  helper components below to a shared file and the folder table to its own
  file.
- `settings/local_library/SettingsPrimitives.slint` (~90 lines) — the
  generic small components currently private to this file but reusable
  across other Settings panels: `GroupHeader`, `SecondaryButton`, `IconBtn`
  (lines 21-86). Consider checking whether other Settings `.slint` files
  already duplicate `GroupHeader`/`SecondaryButton` (they likely do, per
  the file's own comment "Matches OfflineSettings/AudioSettings'
  SecondaryButton") — if so, this is also an opportunity to de-duplicate
  into one shared settings-primitives file all panels import, rather than
  each panel defining its own copy. Flag this for the actual implementer to
  confirm before moving.
- `settings/local_library/FolderTable.slint` (~150 lines) — `
  FolderTableHeader` + `FolderRow` (lines 88-243), the folder list
  rendering block (lines 299-337 of the current file: empty-state text +
  the ScrollView + `for f in LibraryFoldersState.folders: FolderRow`).
  Exposes a new small component e.g. `component FolderList` that the main
  file instantiates, taking no props (reads `LibraryFoldersState`/
  `LibraryManageActions` globals directly, same as today).
- `settings/local_library/ScanProgress.slint` (~70 lines) — lines 340-419:
  the determinate scan-progress block (`Scanning: n/total`, Stop button,
  progress bar, current-file text), as its own component reading
  `LibraryScanState`/`LibraryManageActions`.

## Re-export surface
`LocalLibrarySettings.slint`'s exported `LocalLibrarySettings` component
remains the sole import surface (`import { LocalLibrarySettings } from
"settings/LocalLibrarySettings.slint";` from wherever the Settings view
mounts it, e.g. `SettingsView.slint`) — unaffected by the internal split.

## Coupling / watch out
- `FolderRow`'s column widths (20/stretch/120/84/76 + spacings) MUST stay
  in exact sync with `FolderTableHeader`'s columns — the file's own comment
  already calls this out ("Shared column geometry ... header + rows MUST
  match"). Keep both components in the SAME new file (`FolderTable.slint`)
  so a future edit to one is co-located with the other, not scattered
  across files.
- If `GroupHeader`/`SecondaryButton` turn out to be duplicated verbatim in
  sibling Settings files (OfflineSettings.slint, AudioSettings.slint per
  the comment), moving them to a shared file changes MULTIPLE settings
  panels' imports, not just this one — treat that as a follow-up refactor
  outside this file's scope unless coordinating with whichever other agent
  owns those files, to avoid conflicting edits.
- `IconBtn` is used both in the folder toolbar and inside `FolderRow`'s
  actions column — after the split it must be visible to both
  `LocalLibrarySettings.slint` (toolbar) and `FolderTable.slint`
  (row actions), so it belongs in the shared `SettingsPrimitives.slint`,
  imported by both.

## Verify after split
- App build + `slint-viewer` check of the Settings > Local Library panel.
- Visual/functional smoke test: add/remove/edit/enable a folder, run a
  scan and watch the progress bar + Stop button, change album grouping,
  run Cleanup missing, and exercise the two-step danger-zone Clear.
