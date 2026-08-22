# `crates/qbz-ui/ui/shell/QueueSidebar.slint` (1086 lines)

## 1. Summary
The right-side Queue panel: tab header (Queue/History), a "NOW PLAYING" card
+ paginated "UP NEXT" list with drag-reorder, a "RECENTLY PLAYED" history
list, and a footer with four action icons, a sleep-timer popover, and an
inline search field. Several private helper components (`CloseButton`,
`QueueTab`, `IconButton`, `ExplicitBadge`, `QueueRow`) are defined inline
before the exported `QueueSidebar` component.

## 2. Proposed module layout

New directory `crates/qbz-ui/ui/shell/queue/` holding the extracted pieces;
`QueueSidebar.slint` stays at its current path as the assembly/export point.

- `shell/QueueSidebar.slint` (~150) — imports all pieces below; keeps only
  the outer `Rectangle`, the top-level reorder state properties
  (`reorder-active/from/over/list-top/pointer-y`), `slot-from-pointer`/
  `commit-reorder`, the `init` snapshot pull, and composes
  `QueueHeader` / `QueueBody` / `QueueFooter` / `QueueDragGhost`.
- `shell/queue/QueueControls.slint` (~110) — the small shared leaf
  components: `CloseButton`, `QueueTab`, `IconButton`, `ExplicitBadge`.
  These are generic enough other queue/footer files need them.
- `shell/queue/QueueRow.slint` (~200) — the `QueueRow` component
  (drag-reorder gesture handling, leading number/thumbnail, title/artist,
  duration, per-track context `PopupWindow` menu). Still likely >130 after
  extraction — split the context-menu `PopupWindow` block out into
  `shell/queue/QueueRowContextMenu.slint` (~70) instantiated from
  `QueueRow.slint` (~130), OR accept it as one cohesive interactive unit
  and note it in a follow-up if Slint's component wiring makes further
  splitting awkward (a `PopupWindow` needs to be a direct child to anchor
  correctly — verify during the real split before extracting it).
- `shell/QueueSidebar.slint` header block -> `shell/queue/QueueHeader.slint`
  (~60) — the tab row + close button (currently inline HorizontalLayout
  under "--- Header ---").
- `shell/queue/QueueBody.slint` (~40) — just the tab-switch `Rectangle`
  wrapper choosing between `QueueTabBody` and `HistoryTabBody`.
- `shell/queue/QueueTabBody.slint` (~230, still needs a split) — the Queue
  tab's Flickable: NOW PLAYING card, UP NEXT list + drag-drop-indicator +
  paginator, empty/no-results states. Split further into:
  - `shell/queue/NowPlayingCard.slint` (~90) — the highlighted current-track
    card with favorite toggle.
  - `shell/queue/UpNextList.slint` (~120) — the `for` loop over
    `QueueState.upcoming-page`, the drop-indicator overlay, and the
    paginator controls.
  - `shell/queue/QueueTabBody.slint` becomes the thin composer (~60) that
    wires `NowPlayingCard` + `UpNextList` + empty/no-results `Text` blocks.
- `shell/queue/HistoryTabBody.slint` (~40) — the "RECENTLY PLAYED" Flickable.
- `shell/queue/QueueFooter.slint` (~290, needs a split) —
  - `shell/queue/QueueFooterActions.slint` (~90) — the 4 `IconButton`
    actions (clear/save-as-playlist/infinite-play/sleep-timer trigger),
    minus the sleep-timer popover body.
  - `shell/queue/SleepTimerPopover.slint` (~150) — the `PopupWindow` with
    armed-countdown / idle-preset-picker / custom-minutes / Set button.
    Still borderline >130; split the idle preset-picker `for` loop into
    `shell/queue/SleepTimerPresetList.slint` (~55) if needed.
  - `shell/queue/QueueSearchField.slint` (~55) — the inline search box.
- `shell/queue/QueueDragGhost.slint` (~35) — the floating drag-ghost
  `Rectangle` currently declared as the last root-level child.

## 3. Re-export / public API surface
`crates/qbz-ui/ui/shell/QueueSidebar.slint` stays the single import path —
every other `.slint` file that does
`import { QueueSidebar } from "../shell/QueueSidebar.slint";` (e.g.
`AppShell.slint`) needs zero changes. All new files live under a
`shell/queue/` subfolder and are import-only additions internal to
`QueueSidebar.slint`.

## 4. Tricky coupling / shared state to watch
- `QueueRow`'s drag-reorder gestures (`drag-begin`/`drag-update`/`drag-end`,
  `page-local-index`) are wired to reorder-state PROPERTIES that live on the
  top-level `QueueSidebar` root (`root.reorder-active`, `root.reorder-from`,
  etc.), not on `QueueRow` itself. When `QueueRow` moves to its own file and
  `UpNextList`/`QueueTabBody` become intermediate layers, these callbacks
  must bubble all the way up through each new intermediate component (Slint
  requires each layer to re-declare and forward the callback) — this is the
  single riskiest part of the split; get it wrong and drag-reorder silently
  breaks or drags the wrong row.
- The drop-indicator `Rectangle` inside `UpNextList` reads
  `root.reorder-over`/`root.reorder-from` from the (now more distant)
  top-level root — same bubbling concern.
- The floating drag-ghost (`QueueDragGhost`) reads
  `QueueState.upcoming-page[root.reorder-from].title` and positions itself
  using `root.absolute-position.y` — it must remain a direct, non-layout
  child of the outermost `QueueSidebar` `Rectangle` (a `VerticalLayout`
  child would ignore explicit `x`/`y`), so keep it instantiated at the
  `QueueSidebar.slint` top level, not nested inside `QueueBody`.
- The comment above the Flickable explains `interactive: false` is
  INTENTIONAL (Slint 1.16 drag/scroll interaction bug) — preserve that
  comment verbatim wherever the Flickable ends up (`QueueTabBody.slint`) so
  a future agent doesn't "fix" it back to `interactive: true`.
- Scroll-position restore pattern used elsewhere in this codebase (see
  `MixView.slint`'s `NavState.restore-scope` handling) is NOT present here
  but note the `QueueState.panel-opened()` call in `init` — this must stay
  on the actual root component that gets created/destroyed per ADR-010
  (conditionally mounted panel), i.e. stays in `QueueSidebar.slint`'s `init`,
  not moved into a sub-component that might have different lifecycle
  semantics.
- `SleepTimerState`/`SleepTimerActions` globals are shared with other views;
  no special coupling beyond normal global access.

## 5. What to verify after the real split
- `slint-viewer` (or the project's existing Slint compile check / `cargo
  build -p qbz-ui`) compiles clean — Slint compile errors here are usually
  clear (undefined callback, missing property) so a broken bubble-up chain
  will fail fast.
- Manually exercise (or through existing UI smoke tests if any): opening the
  Queue panel, dragging a row to reorder, switching to History tab, using
  the sleep timer popover (both idle and armed states), and the inline
  search field — these are the interaction paths most likely to regress
  from the split.
- Confirm `AppShell.slint` (and any other importer found via
  `grep -rl QueueSidebar`) still compiles unchanged.
