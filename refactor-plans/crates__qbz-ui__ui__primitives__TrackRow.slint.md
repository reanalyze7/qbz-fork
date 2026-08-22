# crates/qbz-ui/ui/primitives/TrackRow.slint (767 lines)

## Summary
The single shared track-list row used by every track surface (album,
artist, search, favorites, mix, playlist, local library). One component
with a large flag surface (`show-artwork`, `show-album`, `show-favorite`,
`show-download`, `show-source`, `multi-select-mode`, `show-reorder`,
`force-local-menu`, etc.), a drag-source `TouchArea` with reorder/back-
forward-mouse-button handling, and a `HorizontalLayout` of ~10 conditional
column cells, plus the `open-track-menu` function.

## Proposed split
- `TrackRow.slint` (~140 lines) — **stays the public re-export/root
  component**. Keeps `root`'s property list, `row-inert`/`opacity`, the
  `HorizontalLayout` skeleton that composes the cell components below, and
  `open-track-menu`.
- `TrackRowInteraction.slint` (~130 lines, new, internal component or
  mixin) — the drag/click/pointer-event `TouchArea` block (lines 121-244):
  double-click-to-play, multi-select toggle, back/forward mouse nav,
  right-click context menu, and the drag-reorder gesture. Exposes the same
  callbacks (`body-drag-started`, `media-action`) and reads `track`,
  `row-inert`, `draggable`, `show-reorder`, `multi-select-mode` as in-
  properties, wrapping `root`'s children.
- `TrackRowLeadingCell.slint` (~150 lines, new) — the reorder gutter +
  position number + play-cell/checkbox/blacklist-glyph/decrypt-lock stack
  (lines 251-429).
- `TrackRowMetaCell.slint` (~90 lines, new) — title/artist column +
  album column (lines 431-546).
- `TrackRowTrailingCells.slint` (~200 lines, new) — duration, quality
  badge, favorite (+ placeholder), offline-cache button (+ placeholder),
  source glyph, context-menu trigger (lines 548-742).

This is a large, very state-heavy UI component; a simpler fallback split
if the above proves too invasive is just 3 files: interaction (drag/click),
leading cell, and "everything else" (meta + trailing cells + menu
function) — still gets every file under 300 lines even if not all under
130, which the implementer should treat as a first pass, further splitting
trailing cells if still over budget.

## Re-export surface
`TrackRow.slint` must remain the file every other `.slint`/Rust caller
imports (`import { TrackRow } from "primitives/TrackRow.slint"`) — keep it
as the component that owns `root`'s public properties/callbacks
unchanged, just delegating rendering to the new sub-files.

## Coupling / watch out
- Heavy use of `root.` cross-references between cells (row-hover depends
  on `play-cell.hovered` from the leading cell; `menu-hovered` set from
  the trailing "..." button but read by `row-hovered` at the top) — any
  split must either keep these as shared named elements within one file,
  or promote them to `in-out property` bindings threaded through the new
  sub-components. This is the single biggest risk in this split.
- `open-track-menu()` writes many `TrackMenuState` fields and is called
  both from the interaction TouchArea (right-click) and the trailing
  "..." button — keep it defined on `root` (or globally accessible) so
  both callers can invoke it regardless of which file they land in.
- The drag-ghost math (`root.absolute-position.x - root.x + ta.mouse-x`)
  appears 4+ times and must stay byte-identical across call sites if
  duplicated into multiple files — consider a small pure function instead
  of copy-pasting the formula.
- Numerous behavior-preserving comments reference exact Tauri/Svelte
  parity details (e.g. blacklist dimming, explicit-badge measurement
  workaround) — preserve these comments verbatim in whichever file the
  code moves to; they are load-bearing documentation, not decoration.

## Verify after split
- `cargo build -p qbz-ui` (Slint compile).
- Manual smoke test across every surface that reuses `TrackRow`: album
  detail, artist Popular Tracks, search results, favorites, mix, playlist
  detail (multi-select + reorder), local library (show-source). Check
  drag-to-sidebar-playlist, drag-reorder (#589), right-click context menu,
  and the offline-cache/favorite/blacklist row states.
