# crates/qbz-ui/ui/locallibrary/LocalLibraryView.slint (2681 lines)

## Summary
The Local Library view: fixed chrome (title/tab-menu row + per-tab
toolbar row) over a single scrolling content area with four tabs — Albums
(chunked grid), Tracks (server-paginated flat list, incl. the recently-added
client-side "Hi-Res only" toggle), Folders (flat grid/list OR a two-pane
tree browser, plus an "ephemeral folder" pane), and Artists (two-column
master/detail) — plus an Albums quality/format/source filter popup overlay.

## Proposed split
This is the biggest file in the whole `refactor-plans/` batch (2681 lines,
~20x budget). Split by tab (the natural seam this file itself uses via
`if LocalLibraryState.active-tab == "..."` blocks), pulling shared small
components into their own primitive files first:

- `LocalLibraryView.slint` (~120 lines) — the `export component
  LocalLibraryView` shell only: the fixed-chrome VerticalLayout (row 1
  title+gear, divider) delegating row-2 toolbars and tab bodies to
  imported components, plus the filter-popup overlay call. This becomes the
  single import surface every other file already uses.
- `locallibrary/LibTabMenu.slint` (~20 lines) — the `LibTabMenu` component
  (tab-menu SegmentedTabBar wrapper).
- `locallibrary/shared_rows.slint` (~230 lines) — the small reusable pieces
  used across 2+ tabs: `CircleIconBtn`, `ViewToggle`, `ModeToggle`,
  `RailBulkBtn`, `AlphaStrip`, `FilterChip` (all currently standalone
  components with no cross-tab-specific logic).
- `locallibrary/AlbumsToolbar.slint` + `locallibrary/AlbumsTab.slint`
  (~230 + ~200 lines) — the Albums row-2 toolbar (search/sort/group/
  id-mode/filter-badge/view-toggle/multi-select) and the Albums tab body
  (loading/error/empty/no-match states, the chunked grid Flickable +
  AlphaStrip + ListScrollbar). `FolderSubcard` (used by the Folders detail
  pane, not Albums) moves to `shared_rows.slint` or its own file instead.
- `locallibrary/TracksToolbar.slint` + `locallibrary/TracksTab.slint`
  (~150 + ~230 lines) — Tracks row-2 toolbar (multi-select, the Hi-Res-only
  `CircleIconBtn`, search, sort, group) and the Tracks tab body (loading/
  error/empty/no-match/list states, the `ListView` with the Hi-Res filter
  `visible:` binding, group headers, `AlphaStrip`, `ListScrollbar`). The
  `root.tracks-hires-only` property moves from the top-level view into
  whichever component owns both the toolbar toggle and the list's `visible`
  binding — likely `TracksTab.slint` needs to own the property with the
  toggle exposed as a callback from `TracksToolbar.slint`, OR keep the
  property on the top-level `LocalLibraryView` and pass it down as an `in`
  property to `TracksTab` (simpler, avoids a two-way callback wire — this
  is the recommended approach since the toggle is view-level UI state, not
  library state).
- `locallibrary/FoldersToolbar.slint` + `locallibrary/FoldersFlat.slint` +
  `locallibrary/FoldersTree.slint` + `locallibrary/EphemeralPane.slint`
  (~90 + ~140 + ~460 + ~200 lines) — Folders is the biggest sub-split:
  toolbar, flat-mode grid, tree-mode two-pane browser (left rail +
  draggable divider + right detail pane — itself the largest single chunk,
  may need a further split into `FoldersTreeRail.slint` (~200) +
  `FoldersTreeDetail.slint` (~260) if it doesn't fit under 460), and the
  existing standalone `EphemeralPane` component (already well-isolated,
  just needs to move to its own file). `TreeRow` component moves to
  `shared_rows.slint` or a `FoldersTreeRow.slint` file since it's
  tree-mode-specific but substantial (~130 lines alone).
- `locallibrary/ArtistsToolbar.slint` + `locallibrary/ArtistsTab.slint`
  (~20 + ~230 lines) — Artists row-2 search toolbar and the two-column
  master (`LocalArtistRow` list + `AlphaStrip`) / detail (selected artist's
  albums grid) layout. `LocalArtistRow` moves to `shared_rows.slint` or its
  own file.
- `locallibrary/AlbumFilterPopup.slint` (~200 lines) — the whole "Albums
  quality/format/source filter" overlay (`if LibAlbumFilterState.open: ...`
  block at the bottom), which is already visually and logically independent
  of everything else (a floating popup, not part of any tab's layout flow).

## Re-export surface
`LocalLibraryView.slint`'s `export component LocalLibraryView` stays the
ONLY thing imported elsewhere (search the UI tree for `import {
LocalLibraryView }` — likely one call site in the shell/router). All the
new files above are internal to `locallibrary/` and imported only by
`LocalLibraryView.slint` (or by each other, e.g. `AlbumsTab.slint` imports
`shared_rows.slint`'s `AlphaStrip`/`FilterChip`); no external caller needs
to change.

## Coupling / watch out
- **State globals fan-out**: every sub-file needs a subset of
  `LocalLibraryState`, `LocalLibraryActions`, `LibAlbumFilterState`,
  `AppearanceState`, `NavState`, `TooltipState`, `UiFocusState`,
  `ShellState` from `state.slint` — when splitting, each new file
  re-imports only what it uses (Slint has no re-export cost), but keep a
  mental note that these are ALL still one shared global store; splitting
  the `.slint` files does NOT reduce the coupling to `LocalLibraryState`,
  it only reduces file size.
- **Scroll-restore pattern repeats 4x** (Albums/Tracks/Folders-flat/
  Artists-rail Flickables each have their own `restored`/`restore-scroll()`
  /`init`/`changed viewport-height`/`changed viewport-y` block referencing
  a DIFFERENT `NavState.restore-scope` string literal — `"ll:albums"`,
  `"ll:tracks"`, `"ll:folders"`, `"ll:artists"`). When splitting these into
  separate tab files, this boilerplate necessarily duplicates across files
  (already duplicated today) — a good follow-up (out of scope for this
  plan) would be a shared `ScrollRestoreFlickable` wrapper component.
- **`root.tracks-hires-only`** (the recently-added Hi-Res filter) is
  currently a property on the TOP-LEVEL `LocalLibraryView` component,
  referenced by BOTH the Tracks toolbar (the toggle button) AND the Tracks
  tab body (the `visible:` binding on each row) — see "Proposed split"
  above for the recommended approach (keep it on the top-level view, pass
  down as `in property` to `TracksTab`). Get this wrong and the toggle will
  silently stop filtering.
- **`albums-flick`/`tracks-list`/`folders-flick`/`detail-flick`/
  `rail-flick`/`artist-albums-flick` id references**: several callbacks
  reference sibling elements by id within the SAME tab block (e.g. the
  Albums `AlphaStrip.jump` callback writes `albums-flick.viewport-y`) —
  these are all within-tab, so splitting by tab preserves them intact. NO
  cross-tab id references exist (verified by reading the whole file), so
  the tab-by-tab split has no dangling-id risk.
- **The recent bug-fix comment** (around the Tracks `visible:` binding,
  ~line 1464-1474: "An earlier version also set `height:
  self.preferred-height`... corrupted the WHOLE list's layout") is
  load-bearing documentation — must be preserved verbatim in
  `TracksTab.slint` since it explains a real regression that was already
  fixed once.
- **`FolderSubcard`** is used only inside the Folders tree-mode detail pane
  (subfolder grid cards) — despite living near the top of the file next to
  other "shared-looking" components, it is Folders-specific and should move
  with `FoldersTreeDetail.slint`, not into the generic `shared_rows.slint`.

## Verify after split
- Slint compile check (`cargo build -p qbz-ui` or the project's
  slint-viewer smoke command) — confirm every new file compiles and every
  import resolves with no orphaned/duplicate component names.
- Manual UI smoke-test covering all four tabs: Albums (search/sort/group/
  filter popup/grid-list toggle/multi-select), Tracks (search/sort/group/
  Hi-Res-only toggle actually filtering rows), Folders (flat mode
  grid+list, tree mode expand/collapse/select/drag-resize/ephemeral pane),
  Artists (rail search, A-Z jump, master/detail selection) — this is the
  highest UI-surface-area file in the batch so a full manual pass matters
  more than for any other file here.
- Confirm scroll-position restore still works on back/forward navigation
  for all four "ll:*" scopes after the split (the four near-duplicate
  restore blocks are an easy place to introduce a copy-paste string-literal
  mistake, e.g. two tabs both restoring to `"ll:albums"`).
