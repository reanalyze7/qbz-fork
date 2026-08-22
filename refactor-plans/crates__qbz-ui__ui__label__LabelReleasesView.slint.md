# `crates/qbz-ui/ui/label/LabelReleasesView.slint` (335 lines)

Label "See all releases" sub-view: circular label header, toolbar (search / group-by-artist
/ Hi-Res filter / sort / grid-list toggle), states (loading/empty/results), the
`AlbumCollectionView` grid, and load-more pagination.

## Proposed split

- `LabelReleasesView.slint` (~110 lines) — stays the public surface: `export component
  LabelReleasesView`, the outer `Flickable` + scroll-restore, composes the sub-blocks
  below, keeps the `AlbumCollectionView` embed and `ListScrollbar`.
- `label/LabelHeader.slint` (~55 lines) — new component for the circular 180px portrait +
  name/total-albums block (lines ~64-116 of the original).
- `label/LabelReleasesToolbar.slint` (~110 lines) — new component for the title/count +
  search/group/Hi-Res/sort/grid-list toolbar row (lines ~120-232), taking `LabelState`
  bindings directly (it's a global singleton, importable from the new file too) or via
  forwarded callbacks — simplest is to keep using `LabelState`/`LabelActions` directly
  since they're already globals, not props threaded through.
- `label/LoadMoreRow.slint` (~35 lines) — the load-more footer button (lines ~298-322).

## Coupling to flag

- `LabelState`/`LabelActions` are global singletons imported from `../state.slint` — safe
  to import directly in each new sub-component, no prop-threading needed.
- The `AlbumCollectionView` windowing `content-offset: 303px` is a magic number computed
  from the static content above the grid (label header + toolbar + spacers) — if the
  header/toolbar heights change during the split, this offset must be recomputed; flag
  this loudly in a code comment when splitting.

## Verify after split

- Slint compile check.
- Visual smoke test: label header renders, toolbar controls (search/group/hi-res/sort/
  view-toggle) work, load-more pagination still triggers, and grid windowing offset is
  still visually correct (no jump/overlap when scrolling).
