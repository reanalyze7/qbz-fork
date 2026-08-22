# crates/qbz-ui/ui/playlist/PlaylistManagerView.slint (1483 lines)

## Summary
The full Playlist Manager surface: fixed header/toolbar (nav + title + search
+ filter/sort dropdowns + folder/flat toggle + view-mode cycle + New-folder +
counter) over a scrolling FOLDERS section and a PLAYLISTS section with three
view modes (grid / list / tree), all driven by precomputed rows from
`crate::playlist_manager` via `PlaylistManagerState`/`PlaylistManagerActions`.

## Proposed split
By UI region — a `playlist/pm_*.slint` set of sibling files, all consumed by
a slimmed-down `PlaylistManagerView.slint` that keeps the same export name.

- `playlist/pm_toolbar_controls.slint` (~180 lines) — `PmSearch`,
  `PmMenuItem`, `PmToolButton`, `PmIconButton`. The small reusable toolbar
  atoms (search field, dropdown-menu row, labeled tool button, icon toggle).
- `playlist/pm_shared_bits.slint` (~90 lines) — `PmActionButton`,
  `PmLocalBadge`, `PmSectionHeader`. Small cross-cutting pieces used by both
  the folder cards and the playlist rows/cards.
- `playlist/pm_folder_cards.slint` (~185 lines) — `PmFolderCard` (grid mode)
  and `PmFolderChip` (list mode). Both only need `PmActionButton`/
  `PmLocalBadge` from `pm_shared_bits` and `PmFolderIcon` (already its own
  file).
- `playlist/pm_playlist_grid_card.slint` (~155 lines) — `PmGridCard`.
- `playlist/pm_playlist_list_row.slint` (~260 lines) — `PmListRow`, including
  its move-to-folder `PopupWindow`. Largest single component; if it still
  exceeds budget after moving shared bits out, split the popup body into its
  own `PmMoveToFolderMenu` component taking `folders`, `current-folder-id`,
  and a `move-to-folder(string)` callback.
- `playlist/pm_tree_rows.slint` (~140 lines) — `PmTreeFolderRow`,
  `PmTreePlaylistRow`. May need a further trim (each row is simple; splitting
  one into its own file is enough if 140 still runs slightly over).
- `playlist/pm_header.slint` (~200 lines) — new component `PmToolbarHeader`
  extracted from the current inline header block (lines 1089-1263): nav
  buttons + title on the left, search/filter/sort/folder-toggle/view-cycle/
  new-folder on the right. Reads `PlaylistManagerState`/`PlaylistManagerActions`
  directly (both are globals), so it needs no props — just import and drop
  it in.
- `playlist/pm_folders_section.slint` (~90 lines) — new component
  `PmFoldersSection` wrapping the folders-grid/folders-chips block (lines
  1324-1374), parameterized by `folder-w`/`folder-h`/`chip-w`/`chip-h`/`gap`
  (or hardcode the constants locally since they don't vary per call site).
- `playlist/pm_playlists_content.slint` (~210 lines) — new component
  `PmPlaylistsContent` wrapping the PLAYLISTS section header + empty state +
  grid/list/tree bodies (lines 1376-1461), using `PmGridCard`/`PmListRow`/
  `PmTreeFolderRow`/`PmTreePlaylistRow`.
- `playlist/PlaylistManagerView.slint` (~110 lines) — the slimmed main
  export: root properties (`grid-gap`, card/chip dimensions, `next-view-icon`),
  the outer `VerticalLayout` composing `PmToolbarHeader`, the scroll
  `Flickable` wrapping counter + `PmFoldersSection` + `PmPlaylistsContent`,
  and the `ListScrollbar`.

## Re-export surface
`playlist/PlaylistManagerView.slint` stays the file every other `.slint`
imports (`import { PlaylistManagerView } from "../playlist/PlaylistManagerView.slint"`)
— unchanged export name, now a thin composition of the new sibling files.

## Coupling / watch out
- Every extracted piece reads `PlaylistManagerState`/`PlaylistManagerActions`
  as globals directly rather than via props — this is what makes the split
  easy (no prop-drilling needed for state), but means each new file must
  import those globals from `../state.slint` itself.
- `root.grid-gap`, `root.folder-w/h`, `root.chip-w/h/gap`, `root.card-w/h`
  are currently root properties read by nested `if` blocks via `root.*`.
  Once those blocks move into `PmFoldersSection`/`PmPlaylistsContent`
  components, `root.*` must become either component-local constants or
  `in property` parameters passed from the main view — decide once during
  the split and keep it consistent (recommend: re-declare as local
  properties inside each section component, since none of these values are
  customized by callers today).
- `PmListRow`'s move-to-folder popup reads `PlaylistManagerState.menu-folders`
  (a derived/filtered model) directly — keep that dependency when the popup
  is extracted; don't try to pass the folder list as a prop unless also
  passing the live-filtered `menu-folders`.
- The scroll-restore logic (`NavState.restore-scope`, `NavState.scroll-restore`,
  `NavState.report-scroll`) is wired onto `pm-flick` (the `Flickable` id) —
  keep the `Flickable` and its scroll-restore callbacks in the main
  `PlaylistManagerView.slint` file; don't move it into a sub-component or the
  scrollbar wiring (`viewport-y <=> pm-flick.viewport-y`) breaks.
- `ShellState.pointer-in-window`/`pointer-x`/`pointer-y` hover-active
  calculation for the `ListScrollbar` depends on `pm-flick.absolute-position`
  — same reason, keep colocated with the `Flickable`.

## Verify after split
- Slint compile check (`cargo build -p qbz-ui` or whatever triggers the
  `slint-build` macro) — catches import/id resolution errors immediately.
- Manually exercise: search, filter dropdown, sort dropdown, folder/flat
  toggle, view-mode cycle (grid → list → tree), new-folder button, folder
  card/chip open+edit, playlist grid/list card actions (favorite/hide/edit/
  mixtape/reorder), move-to-folder popup, tree row expand/collapse.
- Grep for `PlaylistManagerView` usage across `qbz-ui`/`qbz` to confirm the
  import path is unchanged for whichever shell file mounts it.
- Confirm scroll-position restore still works when navigating away from and
  back to the Playlist Manager (NavState.restore-scope == "playlist-manager").
