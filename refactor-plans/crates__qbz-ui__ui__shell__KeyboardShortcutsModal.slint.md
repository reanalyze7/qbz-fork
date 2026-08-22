# crates/qbz-ui/ui/shell/KeyboardShortcutsModal.slint (260 lines)

## Summary
Read-only keyboard-shortcuts cheatsheet modal: a category-grouped 3-column
list of keybinding rows (label + keycap chip), a scrim + Escape-to-close
FocusScope, and a footer button that hands off to the editable shortcuts
editor.

## Proposed split
Small over-budget file — split the reusable row/group component out from the
modal shell:

- `KeyboardShortcutsModal.slint` (~185 lines) — KEEP as the main file:
  imports (now including `KbGroupBlock` from its new file), `export
  component KeyboardShortcutsModal` (the scrim, FocusScope, header row,
  Flickable body's 3-column layout referencing `KbGroupBlock`, footer row).
- `shell/KbGroupBlock.slint` (~65 lines) — lines 19-78: the `KbGroupBlock`
  component (category header + its keycap rows), exported from its own file
  and imported by `KeyboardShortcutsModal.slint`.

## Re-export surface
`KeyboardShortcutsModal.slint`'s `export component KeyboardShortcutsModal`
remains the only import surface other files use (`import {
KeyboardShortcutsModal } from "shell/KeyboardShortcutsModal.slint";` — e.g.
from `AppShell.slint`). `KbGroupBlock` becomes `export component
KbGroupBlock` in its own file and is imported internally by
`KeyboardShortcutsModal.slint` via `import { KbGroupBlock } from
"KbGroupBlock.slint";` (same directory, so a plain relative import).

## Coupling / watch out
- `KbGroupBlock` takes `in property <KeybindingCategoryGroup> group` — that
  type comes from `../state.slint` (`KeybindingCategoryGroup`), so
  `KbGroupBlock.slint` needs its own `import { KeybindingCategoryGroup }
  from "../state.slint";`.
- The three-column round-robin layout (`KeybindingsState.groups-col1/2/3`)
  lives in the main modal file's body, not in `KbGroupBlock` — don't try to
  move that grouping logic, it stays exactly where it is, just referencing
  the extracted component.
- The focus-timer / Escape-key handling (lines 109-126) is FocusScope-local
  and must stay in the main file.

## Verify after split
- Slint compile check (`cargo build` triggers slint-build, or dedicated
  slint-viewer/lsp check if the repo has one).
- Manually smoke-test: open the cheatsheet with `?`, verify all 3 columns
  render with correct groups/keycaps, Escape closes it, scrim-click closes
  it, X closes it, and "Customize Shortcuts" opens the editable editor.
