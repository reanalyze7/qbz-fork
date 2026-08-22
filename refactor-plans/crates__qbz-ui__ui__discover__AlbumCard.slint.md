# `crates/qbz-ui/ui/discover/AlbumCard.slint` (519 lines)

The largest of the discover cards: 200x246+ album card with hover overlay (genre/year
meta, favorite/play/more), multi-select checkbox, pin badge, award ribbon, source badge
with tooltip, context menu, title/artist/plays footer with quality badge.

## Proposed split

- `AlbumCard.slint` (~130 lines) — stays the public surface: `export component
  AlbumCard`, outer layout, composes the extracted overlay pieces + footer below.
- `discover/OverlayButton.slint` (~50 lines) — same shared-component recommendation as
  flagged in `PlaylistCard.slint`/`ArtistGridCard.slint` plans (this batch): this file's
  `OverlayButton` (lines 25-70, with the `active` prop and back/forward pointer-event
  forwarding) is the superset version — use IT as the canonical shared component the
  other two import.
- `discover/AlbumCardBadges.slint` (~110 lines) — pin badge, multi-select checkbox, award
  ribbon, and source badge + tooltip (lines ~177-370), taking `album: AlbumCardItem`,
  `select-mode`, `show-source-badge` as properties, forwarding `media-action`.
- `discover/AlbumCardMenu.slint` (~70 lines) — the `album-menu` `PopupWindow`/
  `ContextMenu` (lines ~373-434).
- `discover/AlbumCardFooter.slint` (~90 lines) — title/artist/plays + `QualityBadge`
  stack (lines ~439-517).

## Coupling to flag

- Same `OverlayButton` triplication as `PlaylistCard.slint`/`ArtistGridCard.slint` — this
  is the richest version (has `active`), make it the canonical shared component.
- `overlay-on` aggregates hover state from `hover`, `fav-btn`, `play-btn`, `more-btn`,
  `pin-ta` across what would become 2-3 separate files after splitting — each extracted
  sub-component needs to expose an `out property <bool> hovered` (or similar) that the
  parent ORs together, same pattern needed in the other two card files.
- `album-menu`'s position anchors to `more-btn.absolute-position` — if the menu moves to
  a sub-component, `more-btn` must be passed in or the anchor recomputed relative to
  whatever new parent it has.
- The source-badge tooltip writes directly into the global `TooltipState` — keep that
  coupling as-is wherever the badge block ends up.

## Verify after split

- Slint compile check.
- Visual smoke test: hover overlay (genre/year meta, favorite/play/more), multi-select
  checkbox mode, pin badge, award ribbon (press/qobuzissime variants), source badge +
  purchased tooltip, context menu (open/play/play-next/queue/favorite/block), quality badge.
