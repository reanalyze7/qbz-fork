# crates/qbz-ui/ui/shell/TransportControls.slint (222 lines)

## Summary
The now-playing transport button cluster (shuffle / previous / play-pause /
next / repeat / add-menu, plus a Classic-only inline favorite toggle),
extracted from PlayerBar so it can sit in either the New (centre) or Classic
(left) layout column.

## Proposed split
92 lines over budget. Slint components can't be split mid-declaration, but
the file currently defines TWO independent components (`TransportControls`
and the "+ add menu" popup content) that can separate cleanly, plus the
Classic favorite-toggle button is visually and logically standalone.

- `TransportControls.slint` (~110 lines) — stays the export surface: the
  `TransportControls` component with shuffle/previous/play-pause/next/repeat
  buttons and the two child components inlined below by import, unchanged
  callback signature (`media-action`).
- `AddToMenu.slint` (~75 lines, new file, same directory) — the extracted
  `add-menu := PopupWindow { ContextMenu { ... } }` block (lines 100-175:
  Add to library / playlist / queue / play next / add to mixtape / album
  favorite toggle) as its own component `AddToMenuPopup`, taking `track-id`
  and an `album-favorite`/`album-id` pair as `in` properties and emitting
  the same `media-action`-shaped callback so `TransportControls` just wires
  it through.
- `ClassicFavoriteToggle.slint` (~45 lines, new file) — the Classic-only
  inline favorite Rectangle+QbzIcon+TouchArea block (lines 180-221),
  extracted as its own small component taking `has-track` and wired to
  `QueueState`/`TooltipState` directly (it already reads those globals, so
  no new props needed beyond what it already imports).

## Re-export surface
`TransportControls.slint` remains the single import surface — `PlayerBar`
and any layout column that does `import { TransportControls } from
"../shell/TransportControls.slint";` needs no changes. The two new files are
implementation details imported only by `TransportControls.slint` itself.

## Coupling / watch out
- The add-menu popup's `y: -222px` positioning is relative to its anchor
  button inside `TransportControls` — when extracted, keep the anchor
  Rectangle (36x36) in the parent and pass the popup as a child, don't try
  to reposition it independently.
- Both extracted components read global state directly (`NowPlayingState`,
  `QueueState`, `TooltipState`) rather than taking everything as props —
  preserve that pattern instead of over-parameterizing, to match the
  existing Slint style in this codebase (see the file's own header comment
  about "Reads NowPlayingState + QueueState directly").
- `classic-actions` boolean gates the favorite-toggle visibility — keep that
  `if root.classic-actions:` conditional in the parent, not duplicated
  inside the extracted component.

## Verify after split
- `slint-viewer` (or the project's Slint compile check) on the shell UI to
  confirm no compile errors from the new imports.
- Visual smoke-test: New layout (centre transport, grouped add flyout only)
  and Classic layout (left transport, add flyout + inline favorite heart)
  both render and their click paths (`media-action` callback) still fire.
- Grep for `TransportControls` importers (PlayerBar / layout columns) to
  confirm the public component name and its `media-action`/`play-circle`/
  `classic-actions` properties are unchanged.
