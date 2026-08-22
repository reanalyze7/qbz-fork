# `crates/qbz-ui/ui/discover/PlaylistCard.slint` (291 lines)

Playlist card (album card's twin): 200x246, cover collage, hover overlay
(favorite/follow, play, more), pin badge, context menu, title/subtitle stack.

## Proposed split

- `PlaylistCard.slint` (~130 lines) — stays the public surface: `export component
  PlaylistCard`, outer layout, cover + hover scrim + overlay buttons + pin badge.
- `discover/OverlayButton.slint` (~50 lines) — the internal `OverlayButton` component
  (lines 15-47) is duplicated verbatim across `PlaylistCard.slint`, `AlbumCard.slint`, and
  `ArtistGridCard.slint` (all in this same gap-fill batch, differing only in whether
  `active` exists). Strongly recommend consolidating into one shared, exported
  `discover/OverlayButton.slint` (superset with `active` property, default false) used by
  all three, rather than three near-identical private copies. Flag this loudly since it's
  the single biggest simplification across this batch.
- `discover/PlaylistCardMenu.slint` (~90 lines) — extract the `playlist-menu`
  `PopupWindow`/`ContextMenu` block (lines ~189-246) into its own component, taking
  `playlist: SearchPlaylistItem` and forwarding `media-action`.
- `discover/PlaylistCardFooter.slint` (~40 lines) — the title/subtitle/category stack
  (lines ~251-289).

## Coupling to flag

- Same `OverlayButton` duplication issue flagged in the `ArtistGridCard.slint` and
  `AlbumCard.slint` plans (this batch) — implement the shared version once, referenced
  from all three plans.
- `overlay-on` is computed from hover states spread across `hover`, `fav-btn`, `play-btn`,
  `more-btn`, `pin-ta` — if overlay content/menu move to sub-components, thread `hovered`
  properties back up (same pattern as `ArtistGridCard`).

## Verify after split

- Slint compile check.
- Visual smoke test: hover overlay (favorite/follow toggle, play, more menu), pin badge,
  context menu items (play/play-next/queue/favorite/follow/copy) for both owned and
  foreign playlists.
