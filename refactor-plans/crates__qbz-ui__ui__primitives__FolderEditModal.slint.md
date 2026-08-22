# crates/qbz-ui/ui/primitives/FolderEditModal.slint (277 lines)

## 1. Summary

The create/edit-folder modal: name input, icon-preset grid, color-swatch
grid, custom-image picker, a hidden-from-sidebar toggle, and save/delete
buttons — a single `export component FolderEditModal` driven entirely by
the `FolderEditState`/`FolderEditActions` globals (defined in
`state.slint`).

## 2. Proposed module split

The component's body is one long `VerticalLayout` of clearly-delimited
sections (per its own comments: title, name, icon grid, color grid,
hidden toggle, buttons). Split into the root plus one sub-component per
visual section, matching the existing pattern of other primitives
(`QbzIcon`, `QbzToggle` are already separate small components imported
here):

| New file | Owns | ~lines |
|---|---|---|
| `primitives/FolderEditModal.slint` | Root component: the modal scrim, panel sizing/shadow, title bar with close button, and composes the sub-components below in the `VerticalLayout` | ~55 |
| `primitives/folder-edit/FolderNameField.slint` | The name `LineEdit` block (with the hotkey-guard focus probe) | ~25 |
| `primitives/folder-edit/FolderIconGrid.slint` | The icon-preset `for` loop + the custom-image picker swatch | ~75 |
| `primitives/folder-edit/FolderColorGrid.slint` | The color-swatch grid (`swatch-grid` with manual row/column math) | ~45 |
| `primitives/folder-edit/FolderEditButtons.slint` | The hidden-toggle row + Delete/Save button row | ~60 |

## 3. Re-export / public API surface

Only `export component FolderEditModal` matters externally — it's
imported wherever the folder-edit modal is mounted (e.g. the app shell's
overlay layer). The sub-components under `primitives/folder-edit/` are
plain (non-exported, or exported-but-unused-outside) local components
that `FolderEditModal.slint` imports; no other file in the codebase needs
to import them directly.

## 4. Tricky coupling to watch out for

- Every sub-component reads/writes the same two globals
  (`FolderEditState`, `FolderEditActions`) directly rather than through
  passed-in properties — this is consistent with how the rest of the
  Slint UI is written (globals as the state layer, components as pure
  view), so the split does **not** need to thread props through; each
  new file can `import { FolderEditState, FolderEditActions, ... } from
  "../../state.slint";` independently. Confirm this is indeed the
  project's established pattern before introducing prop-threading as an
  unnecessary complication.
- `PmIconPreset` and `PmColorSwatch` struct types (imported from
  `state.slint`) are used as the `for` loop item types in
  `FolderIconGrid.slint` and `FolderColorGrid.slint` respectively — make
  sure both import them from `state.slint` (or from whichever
  `state/*.slint` file owns them post-split, per the state.slint plan in
  this same directory — `PmIconPreset`/`PmColorSwatch` land in
  `state/myqbz_edit_folder_edit.slint` there).
- The icon-preset grid has a hardcoded `preset.id == "heart" ? ... :
  preset.id == "star" ? ...` chain mapping preset ids to `@image-url`
  paths — this stays in `FolderIconGrid.slint` verbatim; it's business
  logic embedded in markup, not something to refactor away as part of
  the line-count split.
- The Save button's enabled/disabled condition
  (`FolderEditState.name != "" && !FolderEditState.busy`) is duplicated
  between the button's `opacity`, its `TouchArea.mouse-cursor`, and its
  `clicked` guard — again, preserve as-is; not a target for this pass.

## 5. What to verify after the real split

- Slint compile check (`cargo build` on `qbz-ui`) succeeds.
- Visual smoke test: open "New folder" and "Edit folder" (existing
  folder) — name input, icon selection, color selection, custom image
  picker, hidden toggle, and Save/Delete all still work exactly as
  before.
- Confirm `import { FolderEditModal } from "primitives/FolderEditModal.slint";`
  (wherever the app shell mounts it) is unchanged.
- Grep for any other `.slint` file importing the modal's internals
  directly (unlikely, but check) — `grep -rn 'FolderNameField\|FolderIconGrid\|FolderColorGrid\|FolderEditButtons' crates/qbz-ui/ui` should show zero hits before the split (confirming nothing external expects these names yet).
