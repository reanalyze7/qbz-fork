# crates/qbz-ui/ui/discover/PinnedCarousel.slint (175 lines)

## Summary
Pinned-items carousel — title + prev/next paging controls + a clipped
sliding track of mixed cards (album/artist/playlist), one fixed 200x246 slot
per item dispatched on `PinnedItem.kind`, with left/right fade-edge
overlays. Fed from `PinnedState`, used by Home/For You "pinned" sections.

## Proposed split
Small over-budget file — extract the standalone `NavButton` component (used
nowhere else in this file's logic, self-contained) to bring the main
component under budget:

- `PinnedCarousel.slint` (~130 lines) — KEEP as the main file: imports (now
  including `NavButton` from its new file), `export component
  PinnedCarousel` (paging properties, title row referencing `NavButton`,
  the clipped viewport with the sliding `HorizontalLayout` + per-item kind
  dispatch, and the two fade-edge `Rectangle`s).
- `discover/NavButton.slint` (~35 lines) — lines 16-47: the `NavButton`
  component (circular prev/next chip), exported from its own file. Note:
  check whether an equivalent nav-button component already exists elsewhere
  in `qbz-ui/ui/primitives/` (e.g. used by other carousels like
  `PlaylistCarousel`/`ArtistCarousel`) — if so, this is likely a near-
  duplicate and the split should point `PinnedCarousel.slint` at the
  EXISTING shared primitive instead of creating a new one, rather than
  adding a second near-identical `NavButton`.

## Re-export surface
`PinnedCarousel.slint`'s `export component PinnedCarousel` remains the only
import surface used elsewhere (Home/For You views). `NavButton` becomes
`export component NavButton` in its own file, imported internally by
`PinnedCarousel.slint`.

## Coupling / watch out
- `NavButton` reads `ShellState.app-background-active` and
  `AppearanceState.app-background-surface-alpha` directly (line 27-28) —
  its new file needs `import { ShellState, AppearanceState } from
  "../state.slint";` (or `../../state.slint` if nested one level deeper).
- **Check for duplication first**: this exact `NavButton` shape (28x28
  circle, hover surface, enabled/disabled tint) is a common carousel-paging
  pattern — grep `qbz-ui/ui/` for other carousel files with a near-identical
  inline nav-button component before extracting a new one, to avoid
  creating the third or fourth copy of the same primitive across the
  40-agent split (a pattern worth flagging to other agents: several
  carousel `.slint` files likely have this same duplicated component).
- The paging math (`per-page`, `page-count`, `step`, `content-width`) is all
  component-local computed `property <>` on `PinnedCarousel` itself and
  must stay there — it's not reusable across carousels since layout
  constants (`card-width`/`card-height`/`gap`) differ per carousel type.

## Verify after split
- Slint compile check.
- Manually smoke-test the Home/For You "Pinned" section: paging left/right
  at the boundaries (buttons disable correctly), mixed album/artist/
  playlist cards rendering in one row, and the fade-edge overlays appearing/
  disappearing correctly as pages change.
