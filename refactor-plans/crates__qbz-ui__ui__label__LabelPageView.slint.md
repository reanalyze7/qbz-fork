# crates/qbz-ui/ui/label/LabelPageView.slint (675 lines)

## Summary
The Label landing page (port of Tauri's `LabelView.svelte`): circular-portrait
header with Follow/Shuffle/overflow + catalog/library toggle, sticky JUMP TO
bar, Popular Tracks list with progressive reveal, Releases/Critics'
Picks/Playlists/Artists/More Labels carousels, a "library" mode (added
tracks+albums), and a full-description modal.

## Proposed split
Slint components split cleanly by extracting self-contained sub-trees into
sibling `.slint` files, imported back into the main view (the file's own
`SectionTitle` local component is the existing precedent for this pattern):

- `LabelPageView.slint` (~140 lines) — becomes the top-level composition:
  imports, the root component's top-level layout skeleton (Flickable +
  page VerticalLayout), wiring the header, the catalog/library switch, the
  sticky JUMP TO bar, the ListScrollbar, and the desc modal, delegating each
  large piece to a new component below. Keeps `open-album`/`open-artist`/
  `media-action` callback signatures unchanged.
- `label/LabelHeader.slint` (~180 lines) — lines 89-269: the circular
  portrait, name/description/read-more, and the Follow/Shuffle/overflow +
  catalog-library `SegmentedTabBar` action cluster. Takes `LabelState`
  reads directly (it's a global, no props needed) and re-exposes
  `media-action`/`show-desc` toggle via a callback back to the parent.
- `label/LabelPopularTracks.slint` (~170 lines) — lines 282-453: the
  loading spinner, Popular Tracks header + play/select/overflow buttons,
  the bulk `MultiSelectBar`, the `for track in top-tracks` `TrackRow` loop,
  and the Load more/View less reveal control. Owns its own
  `preview-count` property (currently on the root — move it here since
  only this block reads it).
- `label/LabelCarousels.slint` (~90 lines) — lines 455-533: the four
  carousel blocks (Releases/Critics'/Playlists/Artists) plus More Labels,
  each already a thin wrapper around a shared `Carousel`/`ArtistCarousel`/
  `PlaylistCarousel` component — group them since they share the same
  "if length > 0: spacer + carousel" idiom and forward `open-album`/
  `open-artist`/`media-action` straight through.
- `label/LabelLibraryTab.slint` (~40 lines) — lines 539-569: the "In
  library" tracks+albums block.
- `label/LabelDescModal.slint` (~70 lines) — lines 609-674: the
  full-description modal, taking `show-desc` as an `in-out property` bound
  back to the parent (or a callback `close()`).

## Re-export surface
`LabelPageView.slint`'s exported `LabelPageView` component stays the single
import surface — every other view that does
`import { LabelPageView } from "label/LabelPageView.slint";` is unaffected.
The new sibling files are internal to `label/` and are NOT re-exported
elsewhere (mirrors how `ArtistPageView` already factors out shared bits).

## Coupling / watch out
- Several sub-blocks read `LabelState`/`ShellState`/`AppearanceState`/
  `NavState`/`DragState` directly as globals rather than via props — this is
  fine to keep as-is post-split (globals are visible from any `.slint`
  file), but don't accidentally turn them into props, which would add
  needless binding boilerplate.
- The sticky JUMP TO bar's Y position math
  (`jump-anchor.absolute-position.y - page-flickable.absolute-position.y`)
  depends on `jump-anchor` and `page-flickable` both being in the SAME
  component tree as the bar — since the bar stays in the top-level
  `LabelPageView.slint`, keep `jump-anchor` (the reservation Rectangle,
  lines 275-277) in the top-level file too, not moved into
  `LabelHeader.slint`.
- `preview-count` (Popular Tracks progressive reveal) is read only within
  the Popular Tracks block — safe to relocate, but confirm no other part of
  the file references `root.preview-count` before moving it.
- `show-desc` is toggled from `LabelHeader.slint` (Read more click) and
  read by `LabelDescModal.slint` — needs to stay as an `in-out property`
  on the root `LabelPageView` (as now) with both children binding to it via
  two-way binding or callbacks; do not duplicate the boolean in two places.

## Verify after split
- `slint-viewer` (or the app's normal build) renders `LabelPageView`
  without warnings; visually diff the label page (header, popular tracks,
  carousels, library tab, desc modal, sticky jump bar) against current
  behavior.
- `cargo build -p qbz-ui` / whichever crate compiles the `.slint` files,
  confirming no import-path errors.
- Manually exercise: Follow toggle, overflow menu, multi-select bulk bar,
  Load more/View less at each step (5→20→50→5), catalog/library toggle,
  jump-tab click + scroll, and the description modal open/close.
