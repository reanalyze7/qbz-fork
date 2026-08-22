# crates/qbz-ui/ui/album/LocalAlbumView.slint (615 lines)

## Summary
Dedicated Local Library album detail view (separate from the Qobuz
`AlbumPageView`): header (cover/title/artist/multi-artist expand), action row
(play/shuffle/edit-tags/add-to-playlist/add-to-mixtape), a local-only
version picker for albums with multiple physical copies, and the track list
with disc dividers and source-aware `TrackRow`s.

## Proposed split
Split into sibling `.slint` files under `ui/album/local_album/` (or flat
alongside, matching this crate's existing convention — check a sibling
directory like `ui/album/` for precedent before deciding flat vs nested):

- `LocalAlbumView.slint` (~230 lines) — KEEP as the main file: imports,
  `export component LocalAlbumView`, the Flickable/page scaffolding, the
  header band, the album header layout (cover + title/artist/multi-artist
  expand + info-line), and the `ListScrollbar`. This stays the public import
  surface.
- `local_album/SourceIcon.slint` (~20 lines) — lines 23-36: the
  `SourceIcon` component (hard-drive vs Qobuz-logo glyph).
- `local_album/VersionPicker.slint` (~110 lines) — lines 39-143: the
  `VersionPicker` component (selected-display + dropdown popup), which
  internally uses `SourceIcon` — import it from its new file.
- `local_album/LocalAlbumActionRow.slint` (~70 lines) — lines 315-375: the
  play/shuffle/edit-tags/add-to-playlist/add-to-mixtape `CircleAction` row,
  extracted as its own component taking callbacks (or reading
  `LocalAlbumActions` directly, matching the existing global-state pattern
  used throughout this file).
- `local_album/LocalTrackListHeader.slint` (~90 lines) — lines 411-537: the
  loading-spinner block + toolbar (quality badge + track search) + column
  header, as one component.
- `local_album/LocalTrackList.slint` (~90 lines) — lines 539-602: the
  disc-divider + `TrackRow` `for` loop, as its own component taking
  `tracks` and forwarding `media-action`.

## Re-export surface
`LocalAlbumView.slint`'s `export component LocalAlbumView` stays the only
thing other `.slint` files import (`import { LocalAlbumView } from
"album/LocalAlbumView.slint";`) — the internal sub-components
(`SourceIcon`, `VersionPicker`, `LocalAlbumActionRow`,
`LocalTrackListHeader`, `LocalTrackList`) are wired together inside
`LocalAlbumView.slint`'s own body via `import` statements pointing at the new
files; none of them need to be exported further.

## Coupling / watch out
- Global state singletons used throughout (`LocalAlbumState`,
  `LocalAlbumActions`, `NowPlayingState`, `NavState`, `DragState`,
  `UiFocusState`, `ShellState`) are read directly from many of the extracted
  components — each new file needs its own `import { ... } from
  "../state.slint";` (relative path depth changes if nested one level deeper
  under `local_album/` — use `../../state.slint` there).
- `root.artists-expanded` and `watched-album-id` (lines 153-157) are
  component-local state on `LocalAlbumView` itself, read only by the header
  block — keep them in the main file, do not try to hoist into a sub-
  component (they'd need `in-out property` plumbing for no benefit).
- The scroll-restore block (`sr-armed`/`sr-restore`, lines 169-179) is
  Flickable-local and must stay in the main file's `flick :=` block — it's
  the mechanism that makes back/forward navigation preserve scroll position.
- `TrackRow`'s `media-action` callback dispatch (lines 587-599) has local-
  specific logic (ignore album/artist clicks, route "play" vs other actions
  differently) — keep this exact routing when extracting `LocalTrackList`,
  don't accidentally simplify it to match the Qobuz AlbumPageView's routing.

## Verify after split
- Slint compile check (however this repo runs it — `cargo build` triggers
  the slint-build macro, or a dedicated `slint-viewer`/`slint-lsp` check if
  present in CI).
- Manually smoke-test: open a local album with a single version, one with
  multiple versions (verify the picker switches track lists correctly), a
  multi-artist compilation (verify the "+N more artists" expand/collapse),
  and a multi-disc album (verify disc dividers + per-disc menu still work).
