# crates/qbz-ui/ui/album/AlbumBookletModal.slint (309 lines)

## Summary
In-app PDF booklet reader modal (scrim + centered card, ADR-009/010 pattern):
header (title/close), a paging/zoom/fit/rotate/download toolbar, and a
Flickable page-image viewport with loading/error overlays.

## Proposed split
Slint components can't be split mid-body, but the file already defines one
small reusable sub-component (`ToolbarButton`) plus three clearly separable
visual regions of the outer `AlbumBookletModal`. Extract the regions as their
own components and compose them back, per the `JumpNavBar.slint` precedent.

- `album/AlbumBookletModal.slint` (~90 lines) — top-level `AlbumBookletModal`
  component: the scrim `Rectangle` + click-outside-to-dismiss `TouchArea`, the
  centered card `Rectangle` sizing, the `FocusScope` + Escape-key handling, and
  composition of the three extracted regions below in its `VerticalLayout`.
  Keep `ToolbarButton` defined here too (it's small, ~30 lines, and only used
  by the toolbar region — could also move with the toolbar, see below).
- `album/booklet-modal/Header.slint` (~40 lines) — the title + close-X
  `HorizontalLayout` (currently lines 100-133). New component
  `BookletModalHeader`, no props needed (reads `BookletState` globally, calls
  `BookletActions.close()` directly) — or, if the project convention prefers
  callback-driven components over direct global-action calls, expose a
  `close()` callback forwarded by the parent (check sibling modals'
  convention, e.g. `AlbumCreditsModal.slint`, for which style to match).
- `album/booklet-modal/Toolbar.slint` (~100 lines) — the paging/zoom/fit/
  rotate/download `HorizontalLayout` (currently lines 141-232), INCLUDING the
  `ToolbarButton` sub-component definition (move it here since it's toolbar-only).
  New component `BookletModalToolbar`, no props (reads `BookletState` directly,
  calls `BookletActions.*` directly, matching the existing style).
- `album/booklet-modal/PageArea.slint` (~65 lines) — the page-viewport
  `Rectangle` with the `page-flick := Flickable` + `Image`, plus the loading
  and error overlay `if` blocks (currently lines 243-304). New component
  `BookletModalPageArea`, no props (reads `BookletState` globally).

## Re-export surface
`album/AlbumBookletModal.slint` stays the single import surface —
`shell/AppShell.slint`'s `import { AlbumBookletModal } from
"../album/AlbumBookletModal.slint";` is unaffected. The three extracted
sub-components are imported and composed only inside `AlbumBookletModal.slint`
itself, not re-exported.

## Coupling / watch out
- The header's close-X and the toolbar's paging/zoom/rotate/download buttons
  ALL call `BookletActions.*` directly (global singleton actions, no callback
  threading needed) — this makes the split mechanically easy since no prop/
  callback wiring is required between the extracted components and the parent,
  just global state reads. Preserve this pattern rather than introducing
  callback forwarding that isn't needed.
- The `fs := FocusScope` with the Escape-key handler MUST stay in the
  top-level `AlbumBookletModal.slint` (it wraps the whole card content) — do
  not let it get split into `PageArea.slint` by mistake, since Escape should
  dismiss the whole modal regardless of which region has visual focus.
- `page-flick`'s `viewport-width`/`viewport-height` derive from
  `BookletState.page-pixel-width/height` in `1px` units — keep this exact
  `Math.max(self.width, ... * 1px)` centering-vs-scrolling logic intact when
  moving to `PageArea.slint`.

## Verify after split
- Slint compile check on `AlbumBookletModal.slint` and its sole importer
  `AppShell.slint`.
- Manual smoke-test: open a Qobuz album with a digital booklet, verify page
  next/prev, zoom in/out, fit-width, rotate, download, loading spinner on
  first open, error state (e.g. by simulating a failed fetch), Escape closes
  the modal, click-outside-scrim closes the modal.
