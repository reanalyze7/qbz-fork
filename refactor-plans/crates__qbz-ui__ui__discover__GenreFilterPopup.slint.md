# `crates/qbz-ui/ui/discover/GenreFilterPopup.slint` (389 lines)

## 1. Summary
The Discover "Filter by genre" popup: a simple grid of parent-genre chips OR
(toggle) an "Advanced view" with search + a 3-level checkbox tree, shared
across the three Discover tabs and driving a re-fetch of the discover index.

## 2. Proposed module layout

New directory `crates/qbz-ui/ui/discover/genre/` for the extracted pieces;
`GenreFilterPopup.slint` stays at its current path as the assembly point.

- `discover/GenreFilterPopup.slint` (~140) — the outer popup `Rectangle`
  (sizing/shadow/border), header (title + close button), the "Remember
  selection" / "Advanced view" toggle row, and the footer "Clear filter" —
  composing the two view components below for the body.
- `discover/genre/GenreTreeRowItem.slint` (~95) — the advanced-view tree row
  component (indent, expand arrow, checkbox, name, count). Note the
  z-order-sensitive comment about the row `TouchArea` needing to be
  declared BEFORE the content so the arrow's `TouchArea` sits on top and
  isn't swallowed (bug #545) — preserve that ordering and comment exactly.
- `discover/genre/GenreCard.slint` (~55) — the simple-grid chip component.
- `discover/genre/GenreAdvancedView.slint` (~90) — the search box +
  scrollable tree `Flickable` (instantiates `GenreTreeRowItem` in a `for`
  loop), shown when `GenreFilterState.advanced` is true.
- `discover/genre/GenreSimpleGrid.slint` (~40) — the manual grid-layout
  `Rectangle` (row/column math over `GenreFilterState.genres`, instantiating
  `GenreCard`), shown when `!GenreFilterState.advanced`.

## 3. Re-export / public API surface
`crates/qbz-ui/ui/discover/GenreFilterPopup.slint` remains the single
import path for callers (search `grep -rl GenreFilterPopup` — currently
`ToggleButton.slint`, `AlbumContextMenu.slint`, `AppShell.slint`); the two
new view components and the two row/card leaf components are internal
implementation details imported only from `GenreFilterPopup.slint` itself.

## 4. Tricky coupling to watch
- Both `GenreAdvancedView` and `GenreSimpleGrid` read/write the SAME
  `GenreFilterState`/`GenreFilterActions` globals (search-query, tree,
  genres, toggle/toggle-expand/search/clear) — no prop-drilling needed since
  these are globals, but don't accidentally introduce local shadow
  properties with the same names inside the new components.
- The manual grid math in `GenreSimpleGrid` (`cell-width`, `rows`, per-chip
  `x`/`y` via `Math.mod`/`Math.floor`) is fragile to reformatting — extract
  it verbatim, don't "simplify" the layout math as part of this split.
- `GenreTreeRowItem`'s arrow-vs-row click ordering (see comment in the row
  component about z-order) is the one place in this file where component
  extraction could accidentally change perceived click behavior if the
  `TouchArea` declaration order is altered during the copy — copy the
  component body verbatim, do not reorder children.
- The popup's `height: content.preferred-height` (hug-content sizing) depends
  on the whole body being inside one `content := VerticalLayout` — verify
  the new `GenreAdvancedView`/`GenreSimpleGrid` components don't introduce
  an extra `Rectangle` wrapper with a fixed height that breaks this
  auto-sizing.

## 5. What to verify after the real split
- Slint compile check (`cargo build -p qbz-ui` / `slint-viewer` on the file)
  succeeds.
- Manually exercise: toggling Advanced view on/off, searching genres,
  expanding/selecting tree nodes, selecting grid chips, and "Clear filter" —
  confirm the popup still auto-sizes correctly in both view modes.
- Confirm the 3 known importers (`ToggleButton.slint`,
  `AlbumContextMenu.slint`, `AppShell.slint`) still compile unchanged.
