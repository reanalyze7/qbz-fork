# crates/qbz-ui/ui/discover/ArtistCarousel.slint (225 lines)

## Summary
The artist-carousel widget: title row + prev/next paging controls + a
clipped, animated sliding track of artist/label cards (grid card, legacy
card, or label card depending on flags) — used by the search results "All"
tab and other artist rails.

## Proposed split

- `ArtistCarousel.slint` (~110 lines) — kept as the public entry point:
  `export component ArtistCarousel`, its properties/callbacks, and the
  `HorizontalLayout` title row + `viewport` block, now importing `NavButton`
  and `ViewAllLink` from their own files instead of defining them locally.
- `discover/carousel/NavButton.slint` (~35 lines) — the private
  `component NavButton inherits Rectangle` (prev/next chevron button),
  extracted verbatim. Not exported outside the carousel family unless another
  carousel-style widget in the same directory already duplicates it (worth a
  quick grep — `Carousel.slint`, mentioned in the file's top comment as using
  "the same paging mechanism", may have its own copy that could consolidate
  onto this shared file instead).
- `discover/carousel/ViewAllLink.slint` (~25 lines) — the private
  `component ViewAllLink inherits Rectangle` ("View all" pill link),
  extracted verbatim.

If `NavButton`/`ViewAllLink` are already deduplicated with `Carousel.slint`'s
copies elsewhere, put them under a shared location (e.g.
`crates/qbz-ui/ui/discover/carousel_controls.slint` exporting both) instead of
nesting a new `carousel/` subdirectory — check for an existing shared file
first before creating one.

## Re-export surface
`ArtistCarousel.slint` remains the only file other `.slint` files import from
(`import { ArtistCarousel } from "discover/ArtistCarousel.slint";` or similar,
matching whatever relative path callers currently use). `NavButton` and
`ViewAllLink` are internal helpers, imported by `ArtistCarousel.slint` but not
re-exported — no external caller references them directly today (they're not
`export component`), so no import path changes for anyone outside this file.

## Tricky coupling / watch out
- `NavButton` and `ViewAllLink` both reference `Theme` (from
  `foundation/semantic-colors.slint`) and `Typography` (`ViewAllLink` only) —
  when moved to their own files they need their own `import` statements for
  these, they currently ride on `ArtistCarousel.slint`'s file-level imports.
- `NavButton` also references `ShellState`/`AppearanceState` (for the
  app-background-active alpha-blend branch) and `QbzIcon` — same import
  requirement.
- `ArtistCarousel`'s `page`/`current-page`/`per-page`/`page-count` properties
  drive both `NavButton`'s `enabled` bindings (in the title row) AND the
  `viewport` animation (`x: -root.current-page * root.step`) — these stay in
  the main file since they're `ArtistCarousel`'s own properties, just make
  sure `NavButton`'s `clicked =>` callbacks (`root.page = root.current-page -
  1` / `+ 1`) still compile referencing `root` (the `ArtistCarousel` instance)
  correctly across the import boundary — this is a normal Slint callback
  wiring pattern (the callback body lives at the call site, not inside
  `NavButton`'s own file), so no change needed there, just confirm after
  extraction.
- The three-way card switch (`LabelCard` / `ArtistGridCard` / `ArtistCard`)
  inside the `for artist in root.artists` loop stays in the main file — it's
  core to what `ArtistCarousel` renders, not a natural extraction target.

## What to verify after the real split
- The project's Slint compile check (`cargo build` will fail if `.slint`
  files don't compile, since they're compiled via `slint-build`/`build.rs`;
  there may also be a dedicated `slint-viewer`/lint step — check
  `crates/qbz-ui`'s `build.rs` or CI config for the exact command).
- Visual smoke test via the `run` skill: open a view that renders
  `ArtistCarousel` (search results "All" tab), confirm paging arrows still
  work, hover/click states on `NavButton` and `ViewAllLink` are unchanged,
  and the sliding animation still runs.
- Grep for `ArtistCarousel.slint` imports across `crates/qbz-ui/ui/` to
  confirm every caller's import path is unaffected (it should be, since the
  export stays in the same file).
