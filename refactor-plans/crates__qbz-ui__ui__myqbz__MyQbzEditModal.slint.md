# crates/qbz-ui/ui/myqbz/MyQbzEditModal.slint (147 lines)

## Summary
A single modal driven by `MyQbzEditState.mode` ("rename" | "description" |
"delete") that renders one of three bodies (name LineEdit, description
TextEdit, delete-confirm text) inside a shared 420px panel + title + Cancel/
Submit footer chrome.

## Proposed split
Only 17 lines over budget — the smallest file in this batch. Split the
per-mode body content out of the shared panel chrome, since that's the one
clean seam (`is-rename`/`is-description`/`is-delete` branches, lines 74-109):

- `myqbz/MyQbzEditModal.slint` (~110 lines) — kept as the export surface:
  header comment, imports, `export component MyQbzEditModal inherits
  Rectangle`, the `is-rename`/`is-description`/`is-delete`/`can-submit`
  computed properties, the backdrop + panel scaffold (title Text + footer
  Cancel/Submit buttons), composing the extracted body component in place of
  the three inline `if` blocks.
- `myqbz/MyQbzEditModalBody.slint` (~45 lines) — `export component
  MyQbzEditModalBody` holding the three `if is-rename / is-description /
  is-delete` bodies (lines 74-109: the LineEdit, TextEdit, and delete-confirm
  Text), taking `is-rename`/`is-description`/`is-delete` as `in` properties
  and forwarding the `accepted =>` submit-on-Enter callback up (or calling
  `MyQbzEditActions.submit-rename()` directly from inside, since it already
  reads the `MyQbzEditState`/`MyQbzEditActions` globals directly — no
  forwarding needed, this file can just import the same globals).

## Re-export surface
`myqbz/MyQbzEditModal.slint` stays the only file the My QBZ hero overflow
menu imports `MyQbzEditModal` from; its exported name and lack of
callbacks/properties (fully driven by the `MyQbzEditState`/`MyQbzEditActions`
globals) are unchanged.

## Coupling / watch out
- `root.can-submit` (used by the Submit button's `enabled:` binding) depends
  on `root.is-rename` and `MyQbzEditState.draft-name` — stays in the root
  file since the footer (which reads it) is not being moved.
- The `guard-focused`/`UiFocusState.text-input-focused` hotkey-guard pattern
  (comment references issue #619: `has-focus` on a std-widgets LineEdit/
  TextEdit doesn't propagate a `changed has-focus` at the use site) is
  duplicated identically for BOTH the rename LineEdit and the description
  TextEdit (lines 80-81, 97-98) — when moving both into
  `MyQbzEditModalBody.slint`, keep this duplicated pattern as-is (it's
  already duplicated in the current file, not introduced by the split); do
  not try to unify it into a shared helper as part of this split, that's a
  separate simplification.
- Given how small the remaining chrome file would be (~110 lines) and how
  tightly coupled `can-submit`/footer/backdrop are, this split is
  low-risk — mostly a mechanical extraction of the three `if` bodies into one
  sibling file.

## Verify after split
- Slint compile check for both files.
- Manual smoke-test: open the My QBZ hero overflow menu, trigger Rename
  (type + Enter-submit + Cancel), Edit description (type + Save + Cancel),
  and Delete (confirm text shows the collection name + Delete button is
  destructive-styled) — confirm all three modes still work identically.
- Grep for `MyQbzEditModal {` importers to confirm the call site is
  unaffected.
