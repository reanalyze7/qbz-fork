# crates/qbz-ui/ui/discover/PlaylistBrowseView.slint (338 lines)

## Summary
The Qobuz Playlists "View all" full-list Slint page: a small local
`FilterRadio` component plus the `PlaylistBrowseView` component (fixed
56px header with search/genre/grid-list tools, an optional single-select
category tag bar, infinite-scroll grid/list body, shared scrollbar, and
genre-filter popup overlay).

## Proposed split
By component/section, matching how sibling Discover views in this crate
are already organized (small standalone sub-components in their own
files, the page component importing them):

- `FilterRadio.slint` (~50 lines) — extract the local `FilterRadio`
  component verbatim into its own file (it's already self-contained: one
  `in property`, one callback, no references to `PlaylistBrowseView`'s
  root). This is directly reusable if any sibling browse view needs the
  same radio-tag look later.
- `PlaylistBrowseView.slint` (~130 lines, the re-export/entry surface) —
  keep the `export component PlaylistBrowseView` root, its properties/
  callback, the VerticalLayout skeleton, and the fixed-header block (56px
  NavButtons+title left cluster, search/genre/view-mode right cluster);
  `import { FilterRadio } from "FilterRadio.slint";`.
- `PlaylistBrowseTagBar.slint` (~55 lines) — extract the "Category tag
  bar" `Rectangle` block (the `if PlaylistBrowseState.tags.length > 0`
  section with its Flickable + `for tag in ...: FilterRadio`) into its own
  component taking whatever `PlaylistBrowseState`/`PlaylistBrowseActions`
  bindings it needs directly (these are globals, so no prop-threading
  needed) — `PlaylistBrowseView.slint` then just instantiates
  `PlaylistBrowseTagBar { }` conditionally.
- `PlaylistBrowseList.slint` (~150 lines) — extract the entire "Scrolling
  list" `Flickable` block (`flick := Flickable { ... }`, including the
  scroll-restore logic, infinite-scroll `changed viewport-y`, loading/
  empty states, the grid/list dual rendering, and the bottom load-more
  spinner) into its own component. It needs `root.pl-w`/`root.pl-h`/
  `root.gap` as `in property`s (or hardcode the same constants locally
  since they're only used here) and must forward `media-action` back up
  via its own callback — `PlaylistBrowseView.slint` wires `flick-height`/
  scrollbar geometry off this sub-component's exposed properties (Slint
  requires exposing `viewport-height`/`viewport-y` as `out property`/
  `in-out property` on the extracted component for the scrollbar binding
  to keep working).
- Genre-filter overlay (backdrop `TouchArea` + `GenreFilterPopup`) and the
  `ListScrollbar` instantiation stay in `PlaylistBrowseView.slint` itself
  (small, and they bind directly to `root`/`flick` geometry that's awkward
  to expose across a component boundary) — or, if `PlaylistBrowseList.slint`
  exposes `viewport-height`/`viewport-y`/`height` as forwarded properties,
  these can move too; start conservative (keep them in the root) and only
  extract further if the root is still over 130 lines after the tag-bar +
  list extraction.

## Re-export surface
`PlaylistBrowseView.slint`'s `export component PlaylistBrowseView` stays
the only public import surface — sibling `.slint` files (the Discover tab
router, wherever `PlaylistBrowseView` is instantiated) already `import {
PlaylistBrowseView } from "discover/PlaylistBrowseView.slint";` and that
import path/component name must not change. `FilterRadio.slint`,
`PlaylistBrowseTagBar.slint`, `PlaylistBrowseList.slint` are new internal
files imported ONLY from within `PlaylistBrowseView.slint` (no other file
should need to import them directly unless the reuse case above
materializes).

## Coupling / watch out
- All state is read through GLOBAL singletons (`PlaylistBrowseState`,
  `PlaylistBrowseActions`, `GenreFilterState`, `GenreFilterActions`,
  `NavState`, `ShellState`, `AppearanceState`) — Slint globals are
  importable/accessible from any file that imports them from
  `../state.slint`, so extracting sub-components does NOT require
  prop-threading state down; each new file just adds its own `import {
  ... } from "../state.slint";` line. This makes the split much simpler
  than a typical prop-drilled UI framework.
- `root.header-h` (computed from `PlaylistBrowseState.tags.length`) is
  used by the `ListScrollbar`'s `y`/`height` binding at the bottom of the
  root — if the tag bar moves to its own file, `header-h`'s definition
  must stay in the ROOT (it needs `PlaylistBrowseState.tags.length`
  either way, so no actual coupling problem, just keep the property in
  `PlaylistBrowseView.slint`, not in `PlaylistBrowseTagBar.slint`).
- The scroll-position restore logic (`sr-armed`, `sr-restore()`, `init =>`,
  `changed viewport-height =>`) inside the `Flickable` is keyed on
  `NavState.restore-scope == "playlist-browse"` — a magic string shared
  with whatever code sets `NavState.restore-scope` when navigating away
  from this view (grep for `"playlist-browse"` elsewhere before touching
  this block) — keep this logic together, don't split the Flickable's
  scroll-restore wiring from its `viewport-y` handler.
- The grid math in the `playlists-grid` Rectangle (`columns`/`rows`
  computed properties, absolute-positioned `PlaylistCard`s) is copied
  from `FavoritesView`'s playlists grid per the inline comment — if this
  exact grid pattern recurs across 3+ browse views, flag it to the agent
  covering those other views as a candidate for a shared
  `AutoGridPositioner`-style component later (out of scope for this split
  alone).

## Verify after split
- Slint compile check (however this repo verifies `.slint` files — e.g.
  `cargo check -p qbz-ui` if `.slint` is compiled via build.rs, or a
  `slint-viewer`/`slint-compiler` invocation) confirms no syntax/import
  errors across the four files.
- `cargo check -p qbz` (or whichever crate embeds `qbz-ui`) to confirm the
  generated Rust bindings for `PlaylistBrowseView`'s callback/properties
  are unchanged (same public surface).
- Manual smoke-test: open Discover, navigate to a Playlists "View all"
  page that has tags (confirm tag bar renders + filters), one without
  tags (confirm header collapses to 56px), scroll to trigger infinite
  load-more, toggle grid/list view mode, open the genre filter popup, and
  confirm the scrollbar tracks the list correctly in both view modes.
