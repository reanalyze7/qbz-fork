# crates/qbz-ui/ui/search/SearchResultsView.slint (806 lines)

## 1. Summary

The full Search results page: a toolbar (nav buttons + title + searchType
filter radios + Hi-Res toggle), a five-tab strip, and a scrollable body
that renders either the "All" dashboard (Most-popular hero + Artists
carousel + Albums carousel + Tracks preview + Playlists carousel) or one
of four per-type grids/lists (Albums grid with viewport windowing,
Tracks list, Artists grid, Playlists grid), each with its own
"Load more" button, plus loading/empty states and a shared scrollbar.

## 2. Proposed module split

| New file | Owns | ~lines |
|---|---|---|
| `search/SearchResultsView.slint` | The exported `SearchResultsView` component only: toolbar, tab strip, the `Flickable`/body wiring, loading/empty states, `ListScrollbar` — imports the extracted sub-components below | ~230 |
| `search/search_tabs.slint` | `SearchTab`, `FilterRadio` — the tab-strip and searchType-radio building blocks | ~80 |
| `search/search_sections.slint` | `SectionHeader`, `LoadMoreButton` — small shared row/section widgets used across the per-type tabs | ~65 |
| `search/search_grids.slint` | `AlbumGrid` (incl. the viewport-windowing logic: hysteresis, sampler Timer, `apply-window`/`desired-first-row`/`desired-last-row`), `ArtistGrid`, `PlaylistGrid` | ~230 |

`AlbumGrid` alone is ~120 lines because of the windowing logic and is
kept whole (splitting the windowing math out of the grid it drives would
scatter tightly-coupled state); `ArtistGrid`/`PlaylistGrid` are simple
wrapping grids without windowing, so they stay in the same file as
`AlbumGrid` for cohesion (all three are "per-type tab grid" components)
without pushing any file over ~230 lines.

## 3. Re-export / public API surface

`search/SearchResultsView.slint` stays the single import surface — it is
the only file with an `export component`. The three new files export
their components (`SearchTab`, `FilterRadio`, `SectionHeader`,
`LoadMoreButton`, `AlbumGrid`, `ArtistGrid`, `PlaylistGrid`) as
non-exported-from-crate-root but importable-by-path components; add
`import { ... } from "search_tabs.slint";` etc. at the top of
`SearchResultsView.slint`. Any other `.slint` file that today imports
`SearchResultsView` from `"../search/SearchResultsView.slint"` is
unaffected — that file still exists at the same path and still exports
the same `SearchResultsView` component.

## 4. Tricky coupling/shared state to watch out for

- `AlbumGrid`'s windowing state (`first-window-row`, `target-first-row`,
  the `Timer`) is read from the top-level `Flickable` (`flick`) via
  `visible-top: -flick.viewport-y; visible-height: flick.height;` at the
  call site inside `SearchResultsView` — moving `AlbumGrid` to its own
  file does NOT change this (properties are still passed in from the
  call site), but the long comment explaining the "POST-LAYOUT SNAPSHOT
  SAMPLING" discipline (must not wire visible-top/visible-height to
  `changed` handlers — re-entrancy panic risk) MUST move verbatim into
  `search_grids.slint`, since it documents a real Slint footgun that a
  future editor of that file needs to see in place.
- `AlbumCard`/`ArtistCard`/`PlaylistCard`/`ArtistGridCard` imports need
  to move from `SearchResultsView.slint`'s import list into
  `search_grids.slint` (they're only used by the grids); check no other
  extracted file also needs them (search_tabs.slint and
  search_sections.slint don't).
- `visible-albums`/`visible-tracks` properties (the Hi-Res-only filtered
  vs raw model selection) are declared as `property <...>` INSIDE the
  `VerticalLayout` in the main component body, not inside `AlbumGrid`
  itself — these stay in `SearchResultsView.slint` and are passed as the
  `albums:`/`track:` binding into the grid components, so no change
  needed there, just confirm the property declarations remain visible at
  their existing call sites after other extractions shift line numbers.
- `SearchState`, `SearchActions`, `Theme`, `Typography`, `Radius` imports
  are needed by nearly every extracted file — duplicate the relevant
  `import` lines rather than trying to share one import list across
  files (standard Slint practice; no single shared import file exists in
  this codebase's convention per the README).

## 5. What to verify after the real split

- Slint compiles: run whatever build/viewer check this repo uses for
  `.slint` files (e.g. `cargo build -p qbz-ui` triggers the Slint
  build-script compile step) — confirm no "component not found" /
  "duplicate import" errors.
- Visually smoke-test: run the app, open Search, verify the All-tab
  dashboard (hero + carousels), each per-type tab (Albums grid still
  windows correctly under fast scroll, Artists grid, Tracks list,
  Playlists grid), the Hi-Res toggle filter, the filter radios, and
  "Load more" on each per-type tab.
- Confirm scroll-position restore (`NavState.restore-scope == "search"`)
  still works after navigating away and back — this logic lives in the
  `Flickable` in the main file and is unaffected by the split, but is
  easy to break if the `Flickable`'s `id` (`flick`) accidentally changes
  during the edit.
