# crates/qbz-ui/ui/favorites/FavoritesView.slint (2071 lines)

## Summary
The Library/Favorites screen: fixed chrome (title + per-tab action buttons row,
tab-menu + per-tab toolbar row) over a single scrolling content area that
switches between 6 tab bodies — "All" (mixed feed grid/list), Tracks
(virtualized ListView), Albums, Artists (grid/grouped/sidepanel), Playlists,
Labels — plus a handful of small standalone helper components used by the
toolbars and the sidepanel.

## Proposed split
By domain (per-tab bodies) + a shared-helpers module, since this is UI
composition, not pure/IO logic:

- `favorites/mod.rs`-equivalent — NOT applicable (Slint has no mod.rs); instead
  `FavoritesView.slint` itself becomes the ~110-line orchestrator/re-export
  surface: imports the split-out components, owns `FavoritesView`'s top-level
  `VerticalLayout` (fixed chrome rows 1+2 stay here since they read
  `FavoritesState.active-tab` to select toolbars) and delegates each tab body +
  the genre-popup overlay to child components.
- `favorites/FavoritesHelpers.slint` (~230 lines) — the small standalone
  components used across toolbars/sidepanel: `FavTabMenu`, `PlaylistSubTab`,
  `SidepanelArtistRow`, `FavGenreButton`, `FavHiResButton`,
  `LibrarySortOption`, `ViewToggle`, `AlphaStrip`. These have no dependency on
  each other beyond shared imports, so one file is fine (~230 lines is over
  budget too — if so, split further into `FavoritesToolbarControls.slint`
  (FavGenreButton, FavHiResButton, ViewToggle, LibrarySortOption — the
  toolbar-only pieces, ~140 lines) and `FavoritesListHelpers.slint`
  (FavTabMenu, PlaylistSubTab, SidepanelArtistRow, AlphaStrip — the
  list/row-oriented pieces, ~150 lines)).
- `favorites/FavoritesToolbarRow.slint` (~140 lines) — row 2's per-tab
  toolbars (search/genre/sort/group/view-toggle for tracks/albums/labels/
  playlists/artists/"all"), extracted from the giant row-2 `HorizontalLayout`.
  Takes `active-tab` and forwards the same `FavoritesActions`/
  `LibraryAllActions` calls it does today.
- `favorites/FavoritesAllTab.slint` (~180 lines) — the "All" mixed-feed tab
  body: loading/error, grid (card-per-kind), and list (compact mixed rows +
  per-kind context menu). This is the single largest sub-block (lines
  944–1438, ~495 lines) — split further into `FavoritesAllGrid.slint` (~140,
  grid view) and `FavoritesAllList.slint` (~200, list view + its
  `row-menu` PopupWindow) if 180 still feels too dense; the context-menu items
  alone are ~70 lines and could become `FavoritesAllRowMenu.slint`.
- `favorites/FavoritesTracksTab.slint` (~120 lines) — the Tracks tab body:
  multi-select bar, empty states, virtualized `ListView` with group headers +
  hi-res filter, `ListScrollbar`, name-grouping `AlphaStrip`.
- `favorites/FavoritesOtherTabs.slint` (~230 lines) — the shared
  `other-flick` Flickable hosting Albums/Artists(grid)/Playlists/Labels tab
  bodies (lines 1617–1845) plus their alpha-strips; likely still over 130, so
  split into `FavoritesAlbumsTab.slint` (~50), `FavoritesArtistsTab.slint`
  (~90, flat + grouped grids), `FavoritesPlaylistsTab.slint` (~80, sub-tabs +
  grid/list), `FavoritesLabelsTab.slint` (~30), each mounted conditionally by
  the parent Flickable which itself can stay in `FavoritesView.slint` or move
  to a thin `FavoritesOtherTabsHost.slint`.
- `favorites/FavoritesArtistsSidepanel.slint` (~175 lines) — the two-column
  sidepanel view (lines 1881–2052): left A-Z artist list + alpha strip, right
  5-state selected-artist album sections. Split into
  `FavoritesSidepanelList.slint` (~70) and `FavoritesSidepanelDetail.slint`
  (~110) if still tight.

## Re-export / import surface
`FavoritesView.slint` stays the only file other `.slint` files import
(`import { FavoritesView } from "favorites/FavoritesView.slint";` — grep
callers, likely `AppShell.slint` / a router). All new sub-components are
internal to the `favorites/` folder; `FavoritesView` re-exports nothing new
(the `export component FavoritesView` signature — `open-album`/`open-artist`
callbacks — is unchanged).

## Coupling / watch out
- Root-level properties `tracks-hires-only`, `artist-w/h`, `artist-card-w/h`,
  `gap` are read by multiple tab bodies (Tracks, Artists, Playlists, Labels) —
  when split into separate components these need to become `in property`
  params passed down from `FavoritesView`, or move to `FavoritesState` if
  truly cross-cutting (simplest: pass as `in property` since they're view-only
  layout constants, not app state).
- `row-rect.menu-x`/`menu-y` in the "All" list view is set from THREE
  different TouchAreas (row body, play button, title, subtitle) that must all
  still reach the same `row-menu` PopupWindow — keep these together in
  whichever file owns the list row.
- `other-flick`/`track-list`/`left-flick` Flickable names are referenced by
  their sibling `AlphaStrip.jump()` callbacks and `ListScrollbar` bindings via
  Slint's `id.property` syntax — these must stay in the SAME file as the
  Flickable they reference (Slint element ids aren't visible across file
  boundaries), so don't split a Flickable from its own AlphaStrip/Scrollbar.
- `NavState.restore-scope`/`scroll-restore` scroll-position-restore logic is
  duplicated 3x (tracks list, other-flick, sidepanel would need it too if
  scrollable) — worth factoring into a reusable low-level property pattern,
  but out of scope for this split (note only).
- `root.open-album`/`root.open-artist` callbacks are called from deep inside
  tab bodies — when extracted to child components, these need their own
  `open-album`/`open-artist` callbacks re-forwarded up to `FavoritesView`'s.

## Verify after split
- `cargo build -p qbz-ui` (Slint codegen compiles all `.slint` files transitively).
- Manual smoke-test via `slint-viewer` (if available) or the app itself:
  Library > Favorites, cycle through all 6 tabs, toggle Artists grid/sidepanel,
  toggle "All" grid/list, verify context menus + alpha-strip jumps still work.
- Grep for other importers of `FavoritesView` to confirm the public
  component name/callbacks didn't change.
