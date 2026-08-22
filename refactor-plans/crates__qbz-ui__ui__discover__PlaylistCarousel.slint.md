# crates/qbz-ui/ui/discover/PlaylistCarousel.slint (180 lines)

## Summary
A paging playlist carousel (title + "View all" link + prev/next chevrons +
a clipped sliding track of `PlaylistCard`s), used by Home's Qobuz Playlists
row and the search results "All" tab — mirrors the standard `Carousel`'s
paging mechanism but over `SearchPlaylistItem` data.

## Proposed split
This file is only 50 lines over budget; split out its two small standalone
sub-components, leaving the paging math + track on the main component:

- `PlaylistCarousel.slint` (~120 lines) — becomes the re-export/entry
  surface: the `export component PlaylistCarousel`, its paging properties
  (`per-page`/`page-count`/`current-page`/`step`/`content-width`), the
  header `HorizontalLayout` (title + view-all-link + nav buttons), and the
  clipped `viewport` with the animated sliding track + the two edge
  gradient fade rectangles. Imports the two extracted components below.
- `discover/PlaylistCarouselNav.slint` (~35 lines) — the `NavButton`
  component (the round prev/next chevron button).
- `discover/PlaylistCarouselViewAllLink.slint` (~30 lines) — the
  `ViewAllLink` component.

Given the total is only 180 lines, an equally reasonable alternative is a
two-way split (fold both small components into one
`PlaylistCarouselControls.slint` file, ~65 lines) if keeping the file count
down is preferred — either satisfies the 130-line cap on `PlaylistCarousel.slint`
itself.

## Re-export surface
`PlaylistCarousel.slint`'s `export component PlaylistCarousel` remains the
only import path other files use (`HomeView.slint` does
`import { PlaylistCarousel } from "PlaylistCarousel.slint";`, and the
search-results view presumably does similarly) — unchanged. `NavButton`
and `ViewAllLink` become internal-only imports.

## Coupling / watch out
- `NavButton` and `ViewAllLink` in this file are named identically (or
  near-identically) to components with the same names/roles in the
  standard `Carousel.slint` — check whether `Carousel.slint` already
  defines its own `NavButton`/`ViewAllLink` (the header comment says
  "Same paging mechanism as Carousel"); if so, consider whether these
  extracted components could/should actually be the SAME shared component
  imported from a common location rather than duplicated — flag this for
  whoever does the real split, since deduplicating here would be a
  larger, cross-file change beyond a mechanical split. For THIS plan
  (mechanical split only), keep them separate per-carousel as they are
  today; do not silently merge with `Carousel.slint`'s versions without
  the owner's sign-off, since subtle styling differences (e.g. this
  `ViewAllLink`'s explicit comment about matching "the standard Carousel's"
  wording) suggest they were already intentionally kept in sync by hand.
- The `PlaylistCard` import (`import { PlaylistCard } from
  "PlaylistCard.slint";`) is used only inside the `for playlist in
  root.playlists: PlaylistCard { ... }` repeater in the main file — no
  change needed there.
- `ShellState`/`AppearanceState` globals are used by both extracted
  components (background alpha blending) and the main file's edge-fade
  rectangles (`ShellState.app-background-active` gates `visible`) — each
  file needs its own `import { ... } from "../state.slint";`.

## Verify after split
- Build the Slint UI (`cargo build -p qbz-ui`) to confirm the extracted
  components compile and import paths resolve.
- Visual smoke-test: Home's Qobuz Playlists row and the search results
  "All" tab's playlist carousel — paging chevrons enabled/disabled at the
  ends, "View all" link, and the edge fade gradients when
  `app-background-active` is off.
