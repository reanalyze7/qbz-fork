# crates/qbz-ui/ui/discover/AlbumCollectionView.slint (346 lines)

## 1. Summary
Renders an album collection's content in one of three modes — flat grid,
flat list, or group-by sections — including a self-contained viewport-
windowing engine (a sampler `Timer` + hysteresis band) for the flat grid;
shared by Discover Browse, Label Releases, and Favorites albums.

## 2. Proposed module split
The `AlbumGrid` internal component (lines 16–192, dominated by the windowing
engine) and the exported `AlbumCollectionView` (lines 194–345, the
grid/list/grouped dispatch) are the natural seam:

| New file | Owns | ~lines |
|---|---|---|
| `discover/AlbumCollectionView.slint` | Stays the re-export/orchestrator: module doc, imports, `export component AlbumCollectionView` — the grouped/flat-grid/flat-list dispatch (lines 194–345) | ~155 |
| `discover/AlbumGrid.slint` | The `AlbumGrid` component's public surface + card-slot `for` loop + column/row math (lines 16–51, 162–192) | ~70 |
| `discover/AlbumGridWindowing.slint` — NOT a separate file (Slint has no free-function modules outside a component); instead extract the windowing engine's PROPERTIES + functions (lines 55–160: `row-stride`, `target-first-row`, `desired-first-row`/`desired-last-row`, `apply-window`, `sample-window`, the `Timer`, `notify-window`) into a **private helper component** `AlbumGridWindowState` that `AlbumGrid` composes internally (a non-visual child component holding just the band state + functions, exposing `first-window-row`/`last-window-row`/`window-changed` outward) | ~115 |

Given Slint's lack of a plain "module" concept (only components/structs are
importable), the practical 2-file split is `AlbumCollectionView.slint`
(orchestration) + `AlbumGrid.slint` (the grid + its windowing engine kept
together, since the windowing state is intrinsically tied to `AlbumGrid`'s
own geometry properties `card-width`/`card-height`/`card-gap` and cannot be
meaningfully separated into a non-visual component without threading all of
those through anyway). If `AlbumGrid.slint` alone is still ~180 lines
(windowing + card loop), that is an acceptable single well-scoped file given
the extensive warning comments are essential documentation, not filler.

## 3. Re-export / public API surface
`discover/AlbumCollectionView.slint` remains the only file other `.slint`
files import (`export component AlbumCollectionView`) — its `in property`
surface (`albums`, `grouped`, `is-grouped`, `view-mode`, `show-source`,
`show-favorite`, `show-source-badge`, `select-mode`, `card-width/height/gap`,
`list-row-gap`, `windowed`, `visible-top/height`, `content-offset`) and
callbacks (`window-changed`, `open-album`, `open-artist`, `media-action`) are
unchanged, so Discover Browse / Label Releases / Favorites keep working with
zero edits. `AlbumGrid` becomes an internal import
(`import { AlbumGrid } from "AlbumGrid.slint";`) used only by
`AlbumCollectionView.slint` itself (both the flat-grid branch and each
grouped section's grid branch already instantiate it identically).

## 4. Tricky coupling / shared-state to watch out for
- The extensive comment block (lines 56–83) documents a REAL, field-hit
  runtime panic class ("Recursion detected" in i-slint-core properties.rs)
  tied to `changed` handlers reading layout-result properties
  (`absolute-position`, `flick.viewport-y`) synchronously during
  `user_init`. This comment — and the "no `changed` handlers, no init-time
  reads" invariant it documents — MUST move verbatim with the windowing
  properties into `AlbumGrid.slint`; it is not decorative, it is the reason
  the sampler-`Timer` pattern exists instead of reactive bindings.
- The grouped-sections code path (lines 265–273) explicitly does NOT window
  (full mounts only) for the same panic-class reason (absolute-position
  based section offsets) — keep that explanatory comment attached to the
  grouped `AlbumGrid` instantiation in `AlbumCollectionView.slint`, since a
  future contributor might otherwise "fix" it by wiring `windowed` there too.
- `AlbumGrid`'s two instantiation sites (grouped-section grid at line 274,
  flat grid at line 296) pass different property subsets (grouped passes NO
  windowing props; flat passes all of `windowed`/`visible-top`/
  `visible-height`/`content-offset`) — after extraction both call sites still
  live in `AlbumCollectionView.slint` and both import `AlbumGrid.slint`, so
  this asymmetry is preserved naturally.
- `notified-columns` tracks column-count changes across resizes to re-key
  windowed indices — a resize-driven column change is a second trigger path
  for `notify-window()` besides the timer tick; don't lose this when
  reorganizing `sample-window()`.

## 5. What to verify after the real split
- `cargo build -p qbz-ui` (Slint compile-time check).
- Manual smoke test: Discover Browse in both grid and list view, toggle
  group-by (label/genre sections) if exposed, scroll a long album grid fast
  enough to exercise the windowing placeholder tiles, and confirm no
  "Recursion detected" panic reappears (this is the exact regression class
  the windowing engine was built to avoid — test on both a cold
  view-restore/startup load AND a warm in-app navigation, since the panic
  was timing-sensitive and only reliably hit on the cold path).
- Confirm Favorites Albums and Label Releases (the other two consumers)
  still render identically post-split.
