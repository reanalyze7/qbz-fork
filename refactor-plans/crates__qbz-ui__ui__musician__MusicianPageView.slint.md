# crates/qbz-ui/ui/musician/MusicianPageView.slint (296 lines)

## Summary
The Musician detail page opened from the artist-network sidebar for
Contextual/Weak-confidence relationships: a small header (avatar + name +
role), an optional "Bands & Projects" chip row, and a paginated "Appears
On" album-appearance grid.

## Proposed split
This file is only modestly over budget; split by extracting the two
reusable pieces (`AppearanceCard` and the grid layout math) so the root
component's own body shrinks under 130 lines.

- `MusicianPageView.slint` (~120 lines) — module doc, imports, `export
  component MusicianPageView`: the outer `Flickable`/scroll-restore
  boilerplate, the header (avatar + name/role), the Bands & Projects chip
  row (small enough to keep inline — ~35 lines), and composition of the
  extracted `AppearancesGrid` + `ListScrollbar`.
- `musician/AppearanceCard.slint` (~65 lines) — the `AppearanceCard`
  component (lines 23-84), exported so the grid file can import it.
- `musician/AppearancesGrid.slint` (~90 lines) — the "Appears On" heading +
  count, loading/empty states, the `appearances-grid` `Rectangle` with its
  responsive-column math (lines 238-255), and the "Load more" row (lines
  257-283). Takes `open-album` as a forwarded callback.

## Re-export surface
`MusicianPageView.slint` stays the single import surface for the shell's
page router (`import { MusicianPageView } from
"./musician/MusicianPageView.slint";` is unaffected); it internally imports
`AppearanceCard` and `AppearancesGrid` from the same `musician/` directory.

## Coupling / watch out
- `card-width`/`card-height`/`card-gap` properties (lines 89-91) are used
  both by the grid's column-count math and by each `AppearanceCard`
  instance's placement (`x`/`y` computed from `appearances-grid.columns`)
  — when `AppearancesGrid` becomes its own component, keep these three
  properties (and the `columns`/`rows` computed properties) inside
  `AppearancesGrid` itself rather than passing them down from the root;
  nothing outside the grid needs them.
- `MusicianState`/`MusicianActions` are Slint globals — no prop-drilling
  needed; `AppearancesGrid` reads `MusicianState.appearances`/`.total`/
  `.loading`/`.load-more-loading` directly and calls
  `MusicianActions.load-more()` directly, same as today.
- `open-album` callback threading: the root's `callback open-album(string)`
  is currently invoked from `AppearanceCard.clicked(id) => { root.open-album(id); }`
  inline in the `for` loop — after extraction, `AppearancesGrid` needs its
  own `callback open-album(string)` that the root wires to its own
  `open-album` (`open-album(id) => { root.open-album(id); }`), a simple
  one-hop forward.
- Scroll-restore logic (`sr-armed`, `sr-restore()`, `flick`'s `init`/
  `changed` handlers, lines 98-109) stays in the root file — same pattern
  as `AlbumPageView.slint`, do not extract.

## Verify after split
- `cargo build` through the Slint build step to confirm compilation.
- Smoke-test: open a musician page (via an artist-network sidebar
  Contextual/Weak relationship row), confirm header renders, Bands &
  Projects chips show when present, the Appears-On grid lays out
  correctly at different window widths (the column math is responsive),
  and Load More still paginates.
