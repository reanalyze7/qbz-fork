# crates/qbz-ui/ui/primitives/PlaylistListRow.slint (153 lines)

## Summary
Compact playlist list row (thumbnail + name + owner/track-count subtitle +
play button + "more" overflow menu), shared by the Favorites Playlists list
view and the Qobuz Playlists browse page; presentation-only, driven by the
`media-action(kind, id, action)` callback.

## Proposed split
Only modestly over budget (153 vs 130). Slint requires a component's body to
live whole in one file, but the "more" overflow menu (the `PopupWindow` +
`ContextMenu` + 4 `ContextMenuItem`s, lines 106-150) is a self-contained,
reusable chunk that can be extracted as its own small component — the
idiomatic Slint way to shrink this file.

- `primitives/PlaylistListRow.slint` (~110 lines) — top-level `PlaylistListRow`
  component: the outer `Rectangle`, the full-row `body-ta` TouchArea, the
  thumbnail + title/subtitle `HorizontalLayout`, the play-button
  `VerticalLayout`/`TouchArea`, and now instantiates the extracted menu
  component in place of the current inline `more-ta`/`row-menu` block.
- `primitives/playlist-list-row/RowMenu.slint` (~55 lines) — new component
  `PlaylistRowMenu` wrapping the "more" `TouchArea` (ellipsis icon) +
  `PopupWindow` + `ContextMenu` + its 4 `ContextMenuItem`s (Play, Play next,
  Add to queue, Add to Library). Takes `playlist-id: string` (in) and
  re-emits a single `action(string)` callback (or reuses
  `media-action(string, string, string)` directly if that's simpler — either
  keeps `PlaylistListRow`'s public callback signature unchanged).

## Re-export surface
`primitives/PlaylistListRow.slint` stays the single import surface — the
Favorites Playlists list view and the Qobuz Playlists browse page (the two
call sites mentioned in the file's header comment) continue
`import { PlaylistListRow } from "...PlaylistListRow.slint";` unchanged; the
new `PlaylistRowMenu` sub-component is only imported and composed internally,
not exported for direct use elsewhere.

## Coupling / watch out
- `root.media-action("playlist", root.playlist.id, "...")` is called from
  THREE places today (body click = "open", play button = "play", and all 4
  menu items = "play"/"play-next"/"queue"/"favorite") — when extracting
  `PlaylistRowMenu`, either pass `playlist-id` in and have the sub-component
  emit `media-action` itself, or bubble a generic action string back up to
  the parent to call `media-action` — pick ONE approach so the callback
  fires exactly once per click, not duplicated.
- `row-hover` (used for the background tint) is computed from THREE
  `TouchArea.has-hover` checks (`body-ta`, `play-ta`, `more-ta`) — if
  `more-ta` moves into the sub-component, `row-hover` either needs the
  sub-component to expose its own hover state back out (e.g. an `out
  property <bool> hovered`) or `PlaylistListRow` accepts a slightly
  different hover definition; check the visual result matches (menu-button
  hover currently tints the whole row).
- The popup's fixed offset (`x: -166px; y: 32px; width: 196px; height:
  150px;`) is positioned relative to the ellipsis TouchArea — this must be
  preserved exactly in the extracted component so the popup still opens in
  the same screen position.
- Icon paths (`../assets/icons/*.svg`) are relative to `primitives/` —
  update to `../../assets/icons/...` in the new
  `primitives/playlist-list-row/` subdirectory file.

## Verify after split
- `slint-viewer` (or the project's Slint compile check) on
  `PlaylistListRow.slint` and both call sites (FavoritesView playlists list,
  Qobuz Playlists browse page).
- Visual smoke-test: row hover tint still covers the whole row when hovering
  the ellipsis button, "more" menu still opens at the same position with all
  4 actions wired (Play / Play next / Add to queue / Add to Library), play
  button still fires "play" directly without opening the menu.
- Confirm no other `.slint` file imports `PlaylistRowMenu` directly (should
  stay internal to `PlaylistListRow`'s composition).
