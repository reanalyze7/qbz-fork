# crates/qbz-ui/ui/playlist/PlaylistView.slint (1527 lines)

## Summary
Playlist detail view: header (cover/collage art + metadata + action row +
sort dropdown), track-list column header, virtualized `TrackRow` list with
drag-to-reorder, docked "Suggested Songs" panel, back-to-top FAB, an
"Upload to Qobuz" confirm modal, and a read-more description modal.

## Proposed split
Split into sibling `.slint` files under `ui/playlist/` (flat, matching the
crate's shallow per-view convention seen in `ui/album/`):

- `PlaylistView.slint` (~120 lines) — KEEP as the main file: imports,
  `export component PlaylistView`, its callbacks/top-level properties, and
  the top-level composition (`page := VerticalLayout` wiring the pieces
  below + the FAB + the two modals). This stays the public import surface.
- `playlist/SortOption.slint` (~40 lines) — lines 23-61: the `SortOption`
  row component used inside the sort popup.
- `playlist/SuggestionActionButton.slint` (~50 lines) — lines 66-113: the
  small ghost icon button shared by the Suggested Songs rows.
- `playlist/PlaylistSuggestionRowView.slint` (~180 lines) — lines 118-297:
  the full suggestion-row component (cover+hover-play, title/artist/album
  links, duration, info/add/dismiss actions). Internally uses
  `SuggestionActionButton` — import it from its new file.
- `playlist/PlaylistHeader.slint` (~330 lines) — lines 446-958: the whole
  header block (cover-box with collage/single/placeholder branches +
  hover overlay, metadata column, action-button row, sort dropdown +
  popup using `SortOption`). Takes callbacks (`play-all`, `shuffle`,
  `toggle-favorite`, `edit`, `media-action`) and exposes `sort-label` (or
  recomputes it internally from `PlaylistState`).
- `playlist/PlaylistReorderLogic.slint` — NOT extractable as a separate
  visual component (see coupling note below); the drag-reorder state/
  functions (lines 340-419) stay on `PlaylistView` itself, OR are hoisted
  into the new `PlaylistTrackList.slint` component since that's the only
  place they're read (see next bullet).
- `playlist/PlaylistTrackList.slint` (~250 lines) — lines 973-1177: the
  column header, bulk-select bar, the virtualized `ListView` of `TrackRow`s,
  the drop-indicator rectangle, and `ListScrollbar`. Owns the drag-reorder
  properties/functions (`reorder-from`, `reorder-over`,
  `reorder-drop-playlist`, `pointer-in-list`, `slot-from-pointer`,
  `update-reorder-slot`, the `changed` handlers) since they only affect this
  component's own list; forwards `reorder-track` upward via a callback
  instead of calling `PlaylistActions` directly if the plan wants it
  side-effect-free, or calls `PlaylistActions.reorder-track` directly to
  match the file's existing pattern of components calling globals directly.
- `playlist/SuggestedSongsPanel.slint` (~170 lines) — lines 1179-1338: the
  docked Suggested Songs section (header, loading/error/list states),
  using `PlaylistSuggestionRowView`.
- `playlist/UploadConfirmModal.slint` (~90 lines) — lines 1372-1455: the
  "Upload to Qobuz?" confirm dialog.
- `playlist/DescriptionModal.slint` (~70 lines) — lines 1457-1521: the
  read-more description modal.
- Back-to-top FAB (lines 1343-1370) is small and shared boilerplate
  identical to other views (AlbumView, library) — leave inline in
  `PlaylistView.slint` unless a shared `BackToTopFab.slint` primitive
  already exists elsewhere in the codebase (worth a grep before the real
  split; if found, reuse it instead of duplicating).

## Re-export surface
`PlaylistView.slint`'s `export component PlaylistView` stays the only thing
other `.slint` files import (`import { PlaylistView } from
"playlist/PlaylistView.slint";`). All new sub-files are imported by
`PlaylistView.slint` itself and wired together in its body; none need
further re-export.

## Coupling / watch out
- `track-list` (the `ListView` inside `PlaylistTrackList.slint`) is
  referenced by name from the top-level FAB visibility condition
  (`track-list.viewport-y < -track-list.visible-height`) and the FAB's
  click handler (`track-list.viewport-y = 0`) — once the list moves into
  its own component, these must either read a forwarded `out property
  <length> viewport-y` / exposed alias from `PlaylistTrackList`, or the FAB
  stays inside `PlaylistTrackList.slint` too (simplest: move the FAB into
  the new file since it only depends on that component's internals).
  Splitting them without wiring this through breaks the FAB.
- The drag-reorder state block is heavily coupled to `DragState` (global)
  AND to `track-list.absolute-position` / `viewport-y` (component-local) —
  the `pointer-in-list`/`slot-from-pointer` functions must live in the same
  file as the `track-list` `ListView` they measure against.
- `bulk-actions-owner` / `bulk-actions-follower` properties (lines 316-328)
  are read by the bulk-select bar inside the track-list block — move them
  together with `PlaylistTrackList.slint`, not left behind in the main file.
- `sort-label` (lines 331-338) is read only by the header's sort-dropdown
  button — move it into `PlaylistHeader.slint` rather than keeping it on
  `PlaylistView` root.
- `show-description` / `show-upload-confirm` booleans are root-level
  properties toggled from the header (action-row buttons) and read by the
  two modals — these must stay accessible: either keep them on
  `PlaylistView` root (modals + header both read/write via `root.xxx`,
  unaffected by extraction since Slint components share the parent's
  `root` only within one file) — meaning if the header becomes its own
  component, it needs an explicit callback (`open-upload-confirm()`,
  `open-description()`) rather than writing `root.show-description`
  directly, since `root` would then refer to the header sub-component, not
  `PlaylistView`.

## Verify after split
- Slint compile check (`cargo build` triggers slint-build, or
  `slint-viewer`/`slint-lsp` if available in CI).
- Manually smoke-test: open a playlist, verify header actions (play/
  shuffle/favorite/pin/follow/copy/cache/upload/edit/select), sort menu
  switching + custom-order drag-reorder (drag a row, verify insertion
  indicator and final order), the Suggested Songs panel (activate/refresh/
  dismiss/add), the upload-confirm modal, the read-more description modal,
  and the back-to-top FAB appearing/scrolling correctly.
