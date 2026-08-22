# `crates/qbz-ui/ui/myqbz/MyQbzShared.slint` (163 lines)

Shared MyQBZ index widgets reused by both `MixtapesView` and `CollectionsView`:
`NewActionBtn`, `ViewToggle`, `MyQbzCard` (grid card), `MyQbzListRow` (list row).

## Proposed split

Four small, fully independent components in one file — splitting purely by component
keeps each file well under 130 lines and matches Slint convention of one-component-per-file
for reusable widgets:

- `myqbz/NewActionBtn.slint` (~25 lines) — the "+ New" circular action button.
- `myqbz/ViewToggle.slint` (~30 lines) — the grid/list segmented toggle.
- `myqbz/MyQbzCard.slint` (~45 lines) — the 208px grid card.
- `myqbz/MyQbzListRow.slint` (~50 lines) — the list row.
- Keep `MyQbzShared.slint` as a thin **re-export barrel**: `import` each from its new file
  and `export { NewActionBtn, ViewToggle, MyQbzCard, MyQbzListRow }` so existing
  `import { NewActionBtn, ViewToggle, MyQbzCard, MyQbzListRow } from "MyQbzShared.slint";`
  lines in `CollectionsView.slint` / `MixtapesView.slint` don't need to change.

## Coupling to flag

- All four components import `CollectionMosaic.slint` and theme/typography foundation
  files — keep those imports in each new file (Slint has no transitive re-export of
  imports, only of exported components).
- Used by at least `CollectionsView.slint` and (per the header comment) `MixtapesView`
  — verify both still resolve after the split via the barrel file.

## Verify after split

- Slint compile check.
- Visual smoke test of Collections/Mixtapes grid and list views (cards, toggle, + button).
