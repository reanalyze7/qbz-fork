# crates/qbz-ui/ui/settings/SandboxSettings.slint (175 lines)

## Summary
Settings > Sandbox page: shown only inside Flatpak/Snap installs, renders the
one-time permission-grant shell commands (Flatpak: exclusive-DAC D-Bus
reservation + NAS filesystem access; Snap: required audio interface
connections + optional removable-media) as copy-to-clipboard command blocks.

## Proposed split
Modest overage (45 lines over budget), driven mostly by the shared
`CommandBlock`/`SecondaryButton` micro-component definitions at the top.
Split by "shared primitives" vs. "per-installer-method content":

- `settings/SandboxSettings.slint` (~55 lines) — kept as the export surface:
  header comment, imports, `export component SandboxSettings inherits
  VerticalLayout` with the two `if SandboxState.install-method ==
  "flatpak"/"snap"` branches, each now just composing the extracted
  sub-components below instead of inlining every row.
- `settings/SandboxCommandBlock.slint` (~75 lines) — `GroupHeader`,
  `RowTitle`, `RowNote`, `SecondaryButton`, `CommandBlock` (lines 25-118,
  the whole shared-primitive block, including the `copied` state + Timer
  logic in `CommandBlock`) — all `export component`s so
  `SandboxSettings.slint` (and potentially future sandbox-adjacent pages) can
  import them.
- `settings/SandboxFlatpakSection.slint` (~35 lines) — the Flatpak
  `VerticalLayout` body (lines 125-151): group header + intro note +
  exclusive-DAC row + NAS-filesystem row, each `CommandBlock`.
- `settings/SandboxSnapSection.slint` (~25 lines) — the Snap `VerticalLayout`
  body (lines 154-174): group header + intro note + required-audio-connections
  row + optional-removable-media row.

## Re-export surface
`settings/SandboxSettings.slint` stays the only file the Settings page
router imports `SandboxSettings` from; its exported name and lack of
callbacks/properties (it's driven entirely by the `SandboxState` global) are
unchanged. The three new files are internal to the Sandbox settings section.

## Coupling / watch out
- `CommandBlock`'s `SandboxState.copy-command(root.command)` call and its
  local `copied`/`copied-timer` state are entirely self-contained per
  instance — no shared state between the Flatpak and Snap command blocks, so
  this split has no coupling risk beyond correct imports.
- `SecondaryButton` here is a LOCAL component named `SecondaryButton`,
  distinct from `primitives/SecondaryButton.slint` used elsewhere in the app
  (confirm by checking for a naming collision) — if
  `primitives/SecondaryButton.slint` already exists with a different API,
  keep this one under a distinct name when extracting (e.g.
  `SandboxSecondaryButton`) to avoid ambiguity for future readers, even
  though Slint scoping means there's no compile error either way since it's
  file-local today.
- `SandboxState.install-method` and `SandboxState.copy-command` (the global)
  must be re-imported (`import { SandboxState } from "../state.slint";`) in
  whichever file ends up calling them — currently only `CommandBlock` calls
  `copy-command`, and only the top-level `SandboxSettings` reads
  `install-method`, so only those two new files need the import.

## Verify after split
- Slint compile check for all four files.
- Manual smoke-test: with `SandboxState.install-method` forced to `"flatpak"`
  then `"snap"` (however the dev harness allows overriding this, e.g. a debug
  env var), confirm both sections render their exact command text and the
  Copy button still flips to "Copied!" for 1.5s per row.
- Grep for `SandboxSettings {` importers (the Settings page) to confirm the
  call site is unaffected.
