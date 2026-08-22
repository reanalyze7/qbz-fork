# `crates/qbz-ui/ui/myqbz/CollectionsView.slint` (237 lines)

My QBZ > Collections index grid/list view — header, empty state, toolbar
(search/sort/kind-filter/view-toggle), and grid/list bodies, driven by `MyQbzState`.

## Proposed split

- `CollectionsView.slint` (~90 lines) — stays the public component/import surface:
  `export component CollectionsView`, the outer `Flickable` + scroll-restore + header +
  `ListScrollbar`. Composes the two extracted sub-blocks below.
- `myqbz/CollectionsToolbar.slint` (~60 lines) — new internal component wrapping the
  toolbar `HorizontalLayout` (search / sort `QbzSelect` / kind-filter `QbzSelect` /
  `ViewToggle`), taking sort-index/kind-index as `in property` and exposing the same
  callbacks (`col-set-sort`, `col-set-kind-filter`, `col-set-view`) up via forwarded
  callbacks so `MyQbzActions` calls stay where they are (in `CollectionsView.slint`) or are
  forwarded through.
- `myqbz/CollectionsBody.slint` (~90 lines) — the empty state + populated grid/list bodies
  (the two big `if` blocks with the `MyQbzCard`/`MyQbzListRow` `for` loops), parameterized
  by the same `card-w`/`grid-gap`/`card-h` properties.

## Coupling to flag

- `sort-index`/`kind-index` are computed from `MyQbzState` at the top of the file — keep
  that derivation in the parent `CollectionsView.slint` and pass down as properties, since
  it's shared logic, not view-specific.
- Nearly identical structure to `MixtapesView.slint` (mentioned in the header comment) —
  if that file is also being split, consider whether toolbar/body components could be
  shared between the two instead of duplicated (check for an existing shared file first).

## Verify after split

- Slint compiles (`cargo build -p qbz-ui` or the project's slint build check).
- Visual smoke test: Collections empty state, populated grid, populated list, and the
  scroll-restore-on-back behavior still work.
