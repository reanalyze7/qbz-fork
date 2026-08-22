# crates/qbz-ui/ui/discover/TrackCard.slint (320 lines)

## Summary
A reduced `AlbumCard`-style grid card for a single track (used in the
Library "All" mixed feed): 200x200 artwork with hover play/favorite/more
overlay, an optional always-visible source badge (local/qobuz), a "more"
context menu, and a title + "Track • Artist" meta row with a quality badge.

## Proposed split
190 lines over budget. The file already has two natural components: the
file-local `OverlayButton` helper and the main `TrackCard`; the artwork
block and the context-menu popup are each large enough to be their own
component too.

- `TrackCard.slint` (~90 lines) — stays the export/import surface: the
  `export component TrackCard` shell (sizing, `overlay-on` state, the
  `VerticalLayout` composing the artwork block + meta row below), plus the
  `media-action`/`clicked` callbacks.
- `OverlayButton.slint` (~50 lines, new file, same directory) — the
  file-local circular hover-action button, unchanged, imported by
  `TrackCard.slint` (and reusable by `AlbumCard`/`PlaylistCard` later if
  they're ever de-duplicated — out of scope for this split, note only).
- `TrackCardArtwork.slint` (~110 lines, new file) — the artwork Rectangle:
  cover image, hover scrim, body TouchArea, the three overlay buttons row,
  the source badge, and the `track-menu := PopupWindow { ContextMenu {...} }`
  block. Takes `track: TrackItem`, `show-favorite`, `show-source-badge` as
  `in` properties and re-emits `media-action`/`clicked`.
- `TrackCardMeta.slint` (~70 lines, new file) — the title/artist/quality-badge
  `HorizontalLayout` (lines 258-318), taking the same `track` property and
  emitting `media-action` for the title/artist click-throughs.

## Re-export surface
`TrackCard.slint`'s `export component TrackCard` stays the only thing other
files import (`import { TrackCard } from "../discover/TrackCard.slint";` in
the Library "All" grid). `OverlayButton`/`TrackCardArtwork`/`TrackCardMeta`
are internal to this directory, imported only by `TrackCard.slint`.

## Coupling / watch out
- `OverlayButton` is described in the file's own comment as "File-local by
  design — every card (AlbumCard/PlaylistCard/ArtistGridCard) carries its
  own copy." If it becomes its own file, resist the urge to also change
  AlbumCard/PlaylistCard to import it in this same pass — that's a separate
  de-duplication decision outside this split's scope.
- The context-menu height is hand-computed (`6 * 33px + 10px`) from the
  number of `ContextMenuItem`s — if `TrackCardArtwork.slint` changes which
  items are conditionally shown (`artist-id != ""`, `album-id != ""`,
  `show-favorite`), the fixed height must stay in sync manually (this is a
  pre-existing fragility, not introduced by the split).
- `root.track.removing` drives the fade-out opacity on the OUTER `TrackCard`
  Rectangle — keep that on the top-level component, not on the extracted
  artwork/meta children.

## Verify after split
- Slint compile check (`slint-viewer` or project's build) on the Library
  "All" mixed-feed view.
- Visual smoke-test: hover overlay (favorite/play/more), the "more" context
  menu items (play / play-next / queue / go-to-artist / go-to-album /
  favorite), the source badge, and the title/artist click-throughs all still
  fire `media-action` with the same `(kind, id, action)` triple as before.
- Grep for `TrackCard` importers to confirm the public component name and
  its properties (`track`, `show-favorite`, `show-source-badge`, `card-height`)
  are unchanged.
