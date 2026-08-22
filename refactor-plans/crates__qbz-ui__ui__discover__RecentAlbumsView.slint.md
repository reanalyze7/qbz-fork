# crates/qbz-ui/ui/discover/RecentAlbumsView.slint (172 lines)

## Summary
Full-page "Recently Played Albums" listing opened from the Home rail's
"View all": a fixed header (nav buttons + title) over a scrolling
Flickable grid of the local play-history album store, with scroll-position
restore and a shared right-gutter scrollbar — deliberately simpler than the
catalog-listing views (no pagination/search/error-retry).

## Proposed split
Slint components can't be split into "files that must each define a whole
component" the way Rust modules split functions — the idiomatic move here
is to extract the two visually/logically distinct sub-trees (the fixed
header, and the scrolling content body) into their own small components
that `RecentAlbumsView` composes. This mirrors how other multi-part Slint
views in the codebase are already broken up (see sibling `.slint` files
using a `XxxHeader.slint` + `XxxBody.slint` pattern if present elsewhere in
`discover/`).

- `RecentAlbumsView.slint` (~50 lines) — kept as the public component and
  entry point: root `Rectangle`, the `open-album`/`open-artist`/
  `media-action` callbacks, the `album-w`/`album-h`/`gap` properties,
  composes `RecentAlbumsHeader` + `RecentAlbumsBody` + the `ListScrollbar`,
  and forwards callbacks between them. **This stays the import path every
  other `.slint` file uses.**
- `RecentAlbumsHeader.slint` (~35 lines) — the fixed 56px header: the
  `NavButtons` + "Recently Played Albums" title `Rectangle`/`HorizontalLayout`
  block (lines 40-66 today). No state of its own beyond what it already
  reads from `NavButtons`/`Theme`/`Typography`.
- `RecentAlbumsBody.slint` (~110 lines) — the `Flickable` + scroll-restore
  logic + loading/empty/grid branches (lines 69-152 today), re-exposing the
  `open-album`/`open-artist`/`media-action` callbacks and exposing its
  `flick` element's `viewport-height`/`viewport-y`/`height` as properties
  (or via a `public property` alias) so the parent can still wire the
  `ListScrollbar` against it — this is the one tricky bit, see below.

If the reviewer prefers not to introduce new components for a 172-line
file (only ~1.3x over budget), an acceptable alternative is to leave it as
one file and treat this as a low-priority split — flag this file as a
borderline case rather than a load-bearing violation, but a plan is
provided per the task's instructions.

## Re-export/import surface
`RecentAlbumsView.slint`'s exported `component RecentAlbumsView` is the
name every other `.slint` file already does `import { RecentAlbumsView }
from "discover/RecentAlbumsView.slint";` for (e.g. wherever it is routed to
from the shell's page switcher) — that import path and component name must
not change. `RecentAlbumsHeader`/`RecentAlbumsBody` are new internal
components, imported only by `RecentAlbumsView.slint` itself; they do not
need to be `export`ed unless another view wants to reuse the header pattern
later.

## Coupling / watch out
- The `ListScrollbar` at the bottom of the current file binds directly to
  `flick.viewport-height` / `flick.viewport-y` / `flick.height`, where
  `flick` is the `Flickable` defined inside the (to-be-extracted) body
  component — after the split, `RecentAlbumsView.slint` needs the
  scrollbar to bind through properties `RecentAlbumsBody` exposes (e.g.
  `body.viewport-height`, two-way `viewport-y <=> body.viewport-y`), not
  through a no-longer-visible `flick` id. Get this property-forwarding
  right or the scrollbar silently stops tracking.
- The scroll-position restore logic (`sr-restore`, `NavState.restore-scope
  == "recent-albums"`, `NavState.report-scroll`) is tightly coupled to the
  `Flickable`'s own `viewport-y`/`viewport-height` — keep this logic
  entirely inside whichever component owns the `Flickable` (the body), not
  split further.
- The header's nav-button vertical centering math (`y: 25px - self.height
  / 2`) is pixel-precise ("keeps the same screen position as every page")
  — copy it verbatim into `RecentAlbumsHeader.slint`, don't recompute.
- `AlbumCollectionView`'s `windowed`/`visible-top`/`visible-height`/
  `content-offset` properties are wired directly off `flick.viewport-y` /
  `flick.height` and the page's own `padding-top` (8px) — this viewport-
  windowing wiring must stay inside the body component alongside the
  `Flickable` it measures.
- Reuses `discover.recentlyPlayedAlbums`-family msgids from the Home rail
  (per the file's own comment) — no new strings should be introduced by
  the split; `@tr(...)` calls must move with their exact source text.

## What to verify after the real split
- The Slint compiler / `slint-viewer` (or however this repo's build
  validates `.slint` files — check for a `build.rs` codegen step) accepts
  the new files with no missing-import or property errors.
- Manual smoke: navigate to Recently Played Albums from the Home rail's
  "View all", confirm the header renders identically, the grid scrolls,
  the scrollbar tracks the scroll position, and navigating away and back
  restores the scroll offset (`NavState` round-trip).
