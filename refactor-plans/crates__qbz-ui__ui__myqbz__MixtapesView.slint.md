# crates/qbz-ui/ui/myqbz/MixtapesView.slint (222 lines)

## Summary
My QBZ > Mixtapes index/landing grid: header with "+ New" CTA, empty state,
toolbar (search/sort/view-toggle), and grid/list body rendering
`MyQbzState.mixtapes`, reusing shared `MyQbzCard`/`MyQbzListRow`/
`NewActionBtn`/`ViewToggle` primitives from `MyQbzShared.slint`.

## Proposed split
Only marginally over budget (222 vs 130) — a two-way split by "empty state
vs. populated body" is enough, mirroring the file's own `EMPTY STATE` /
`POPULATED` banner comments (lines 90, 120):

- `MixtapesView.slint` (~90 lines) — the root component: property
  declarations (`card-w`, `grid-gap`, `card-h`, `sort-index`), the
  Flickable + scroll-restore wiring, the header block (title + "+ New"),
  and composition of the two extracted pieces below plus the
  `ListScrollbar`.
- `myqbz/MixtapesEmptyState.slint` (~35 lines) — lines 90-118: the
  `CollectionMosaic` + "No mixtapes yet" text + CTA button, as its own
  component with no props (reads `MyQbzState`/`MyQbzActions` globals
  directly, same as today).
- `myqbz/MixtapesBody.slint` (~100 lines) — lines 120-209: the toolbar
  (search/sort/view-toggle) + search-empty text + grid rendering + list
  rendering, taking `card-w`/`grid-gap`/`card-h`/`sort-index` as `in`
  properties from the parent (since those are computed on the root today).

## Re-export surface
`MixtapesView.slint`'s exported `MixtapesView` component remains the single
import surface for whatever mounts it from the My QBZ shell/router — no
external caller needs to change its `import { MixtapesView } from
"myqbz/MixtapesView.slint";` line.

## Coupling / watch out
- `card-w`/`grid-gap`/`card-h` are used ONLY inside the grid body (lines
  186-199) — after the split these become `in property`s on
  `MixtapesBody`, computed once on the root and passed down; keep the
  literal values (208px/20px/the card-h formula) on the root as the single
  source of truth so `MyQbzCard`'s actual rendered height and this
  computed `card-h` never drift apart (the file's own comment already
  flags this: "card-h matches MyQbzCard's computed height").
- `sort-index` is derived from `MyQbzState.mix-sort` on the root — either
  keep the derivation on the root and pass the int down, or move the whole
  property into `MixtapesBody` and read `MyQbzState` directly there
  (simpler, since `MyQbzState` is a global anyway) — prefer the latter to
  avoid a redundant prop-threading exercise.
- The scroll-position-restore logic (`NavState.restore-scope == "mixtapes"`)
  must stay on the root's `Flickable`, since it needs `self.viewport-y`
  in scope — do not try to extract this into either child component.

## Verify after split
- App build / `slint-viewer` check of `MixtapesView`.
- Visual/functional smoke test: empty state CTA, "+ New" from the header,
  search filter, sort dropdown, grid/list toggle, card click routing, and
  scroll-position restore on back-navigation.
