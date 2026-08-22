# `crates/qbz-ui/ui/discover/ArtistGridCard.slint` (328 lines)

Artist card (200x246) sized to match AlbumCard/PlaylistCard for mixed carousels: round
portrait with hover overlay (follow/play/more), pin badge, and name/subtitle footer.

## Proposed split

- `ArtistGridCard.slint` (~120 lines) — stays the public surface: `export component
  ArtistGridCard`, the outer layout structure, composes the extracted pieces below.
- `discover/OverlayButton.slint` (~50 lines) — extract the internal `OverlayButton`
  component (lines 25-69) into its own file. The file comment already notes this is
  "copied from AlbumCard" with no shared primitive — this split is a good opportunity to
  make it an actually-shared component (`export`ed and imported by both `ArtistGridCard`
  and `AlbumCard`/`PlaylistCard`) instead of triplicated code. Flag this as a
  simplification opportunity, not just a line-count fix.
- `discover/ArtistCardOverlay.slint` (~90 lines) — the hover overlay content: scrim,
  follow/play/more buttons row, and the `artist-menu` `PopupWindow`/`ContextMenu` (lines
  ~137-241), taking `artist: SlimItem`, `follow-mode`, `follow-kind` as properties and
  forwarding `clicked`/`media-action` callbacks up.
- `discover/ArtistCardFooter.slint` (~50 lines) — the name+subtitle footer block (lines
  ~284-327).

## Coupling to flag

- `OverlayButton` is duplicated verbatim across `ArtistGridCard.slint`, `AlbumCard.slint`,
  and `PlaylistCard.slint` (all three are in this same gap-fill batch) — strongly consider
  a single shared `discover/OverlayButton.slint` used by all three rather than three
  separate copies-with-different-names. Note this in whichever plan gets implemented first
  and cross-reference the others.
- `pin-badge` reads `PinnedActions.toggle-pin("artist", ...)` with a hardcoded "artist"
  kind (explicitly NOT `follow-kind`) — keep that hardcoding intact if pin-badge moves to
  a sub-component.
- `overlay-on` is computed from hover states across `hover`, `follow-btn`, `play-btn`,
  `more-btn`, and `pin-ta` — all of which currently live in different depth levels; if
  overlay content and pin-badge move to separate files, `overlay-on` needs `hovered`
  properties exposed from each sub-component (already partly true via `OverlayButton.hovered`).

## Verify after split

- Slint compile check.
- Visual smoke test: artist card hover overlay (follow/play/more + context menu), pin
  badge toggle, and name/subtitle rendering in a mixed carousel with AlbumCard/PlaylistCard.
