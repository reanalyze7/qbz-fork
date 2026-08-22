# crates/qbz-ui/ui/discover/Carousel.slint (301 lines)

## Summary
Discover-section carousel: title/page-chevron header, a horizontally
Flickable track of `AlbumCard`s with slice-based pagination, and an
always-on horizontal-windowing scheme (only cards near the visible band
mount a real `AlbumCard`; the rest are placeholder-slot `Rectangle`s) driven
by an 80ms polling `Timer`.

## Proposed split
By responsibility (small leaf components / windowing math / the main
component's header+track):

- `discover/carousel/NavButton.slint` (~35 lines) — lines 15-46: the
  chevron/refresh circular button component.
- `discover/carousel/ViewAllLink.slint` (~25 lines) — lines 48-70: the "View
  all →" link component.
- `discover/carousel/Carousel.slint` (~200 lines, still over 130 — see
  below) — lines 72-301: the main `Carousel` component itself: imports the
  two components above, keeps the pagination properties (`per-page`, `step`,
  `content-width`, `max-scroll`), the windowing state/functions
  (`desired-first-col`, `desired-last-col`, `apply-window`, `sample-window`,
  the 80ms `Timer`), the header `HorizontalLayout` (title + sort/view-all/
  refresh/chevrons), and the `Flickable` track with its windowed
  `for album[i] in ...` loop.
  - If the reviewer wants this strictly under 130, split further:
    `discover/carousel/Windowing.slint` cannot hold pure Slint properties/
    functions without a component wrapper in current Slint syntax, so instead
    keep the windowing logic inline but shorten by moving the header
    `HorizontalLayout` (lines 185-251, ~65 lines: title + sort dropdown +
    view-all + refresh + chevrons) into its own
    `discover/carousel/CarouselHeader.slint` component, parameterized by
    `in property <string> title`, `in property <bool> show-sort/show-view-all/
    show-refresh`, `in property <[string]> sort-options`, `in property <int>
    sort-index`, `in property <bool> can-page-left/right`, and callbacks
    `sort-changed`, `view-all-clicked`, `refresh-clicked`,
    `page-left-clicked`, `page-right-clicked`. This leaves `Carousel.slint`
    at ~140 lines (windowing + Flickable track only) — close enough that the
    reviewer can decide whether one more small extraction (e.g. the
    `card-slot` placeholder/AlbumCard windowing Rectangle, ~25 lines, into
    `discover/carousel/CardSlot.slint`) is worth the added indirection.

## Re-export surface
`discover/Carousel.slint`'s exported `Carousel` component (the file itself,
or `discover/carousel/Carousel.slint` if the directory-per-file convention is
used) stays the only import path other Discover views use (e.g.
`import { Carousel } from "./Carousel.slint";` from the Discover home view
and Discover Recommendations view). `NavButton`, `ViewAllLink`, and any new
`CarouselHeader`/`CardSlot` are internal-only.

## Coupling / watch out
- **Performance flag for the reported Discover freeze**: the windowing
  `Timer { interval: 80ms; running: root.count > 0; }` runs continuously
  (not just during scroll) for as long as ANY carousel with `count > 0` is
  mounted — every Discover row is a separate `Carousel` instance, so a home
  page with e.g. 6-8 rows means 6-8 independent 80ms timers ticking forever
  while Discover is open, each calling `sample-window()` which reads
  `flick.viewport-x`/`flick.width` and does two `Math.floor` divisions. This
  is comment-documented as intentional ("POST-LAYOUT SNAPSHOT SAMPLING...
  Recursion detected" — a real footgun class in this codebase), so it must
  NOT be "simplified away" during the split — but it IS a plausible
  contributor to a cumulative Discover-page CPU/redraw cost if many rows are
  mounted simultaneously. Worth flagging to whoever investigates the
  reported freeze: check whether `HomeSkeleton.slint`'s own per-row 900ms
  timers (see that file's plan) plus every real `Carousel`'s 80ms timer are
  BOTH alive at once during the loading→loaded transition.
- The windowing state (`target-first-col`, `first-window-col`, etc.) is
  explicitly documented as "PLAIN properties written only by the sampler
  Timer" — never wire them to `changed` handlers or reactive bindings (the
  comment cites a re-entrancy panic risk from `ensure_updated`). Any split
  must keep `sample-window`/`apply-window`/the two `desired-*-col` pure
  functions and the properties they read/write in the SAME component (they
  cannot be factored into a separate non-component Slint module — Slint has
  no free-standing function files, only components).
- `card-width`/`card-height`/`gap`/`col-stride` are read both by the
  pagination math (`per-page`, `step`, `content-width`) and the windowing
  math (`col-stride`, `desired-*-col`) — if `CarouselHeader.slint` is split
  out it does NOT need these (header has no card geometry), but do not
  accidentally split pagination math away from windowing math into separate
  files — they share `content-width`/`max-scroll` and belong together.

## Verify after split
- `slint-viewer` / project slint compile check.
- Full app build.
- Manually open Discover, scroll a carousel with the mouse and via the
  chevron buttons, confirm cards still mount/unmount correctly at the scroll
  edges (no missing covers, no placeholder flash beyond the documented one
  tick), and specifically watch CPU usage while multiple rows are visible to
  see if the described freeze reproduces.
