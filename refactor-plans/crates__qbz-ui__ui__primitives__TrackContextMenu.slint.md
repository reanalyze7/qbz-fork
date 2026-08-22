# crates/qbz-ui/ui/primitives/TrackContextMenu.slint (212 lines)

## Summary
The flat track-row "⋯" context menu (ported from Tauri's TrackMenu): a
`PopupWindow` with ~14 conditionally-rendered `ContextMenuItem`s (play/queue,
favorite/mixtape/playlist, remove-from-playlist, Qobuz share pair, offline
cache state machine, go-to album/artist, track info, reveal-in-explorer),
gated by ~11 boolean `in` properties plus a computed `height` formula and a
`nav-only` reduced mode.

## Proposed split
This is a single flat `PopupWindow` component; Slint components can't easily
be split mid-body without introducing sub-components, so the split is: (1)
extract the height-computation formula into a clearly separated block, and
(2) extract the item list into 2-3 grouped sub-components composed inside the
`PopupWindow`, each importable/exportable but used only from here:

- `primitives/TrackContextMenu.slint` (~55 lines) — kept as the export
  surface: the header comment, `export component TrackContextMenu inherits
  PopupWindow`, ALL the `in property` declarations (track-id, qobuz-actions,
  favorite-action, mixtape-action, playlist-action,
  remove-from-playlist-action, track-info-action, go-album-action,
  go-artist-action, reveal-in-explorer-action, nav-only, cache-status), the
  computed `show-track-info` property, the `media-action` callback, `width`,
  and the `height` formula (lines 74-88, since it needs visibility into every
  property above) — then composes the two extracted item-group components
  below inside its `ContextMenu { ... }` body, forwarding `media-action` and
  the relevant gating properties to each.
- `primitives/TrackContextMenuTransientItems.slint` (~70 lines) — a
  `component` (not necessarily exported outside this pair, but Slint requires
  `export` for cross-file use, so `export component
  TrackContextMenuTransientItems`) holding: Play now / Play next / Add to
  queue / favorite / mixtape / playlist / remove-from-playlist (lines 91-142)
  — the "core transient + list-membership" actions, each still individually
  gated by its own `in property` (re-declared: `nav-only`, `favorite-action`,
  `mixtape-action`, `playlist-action`, `remove-from-playlist-action`) and a
  `media-action` callback forwarded up.
- `primitives/TrackContextMenuQobuzItems.slint` (~90 lines) — the Qobuz-block
  items: share-qobuz / share-songlink / offline cache-status-aware block
  (make-available / refresh+remove) / go-to-album / go-to-artist / track-info
  / reveal-in-explorer (lines 143-210), gated by `in property`s
  (`qobuz-actions`, `nav-only`, `cache-status`, `go-album-action`,
  `go-artist-action`, `show-track-info` — this one is COMPUTED in the root
  since it depends on `track-info-action` AND `(nav-only || qobuz-actions)`,
  so pass it down as a plain `in property <bool> show-track-info` rather than
  recomputing it here, to avoid duplicating the formula).

## Re-export surface
`primitives/TrackContextMenu.slint` stays the only file every track row
(TrackRow, playlist rows, etc.) imports `TrackContextMenu` from — its
exported name, `in` property list, and `media-action` callback signature are
unchanged; the two new sub-files are internal implementation detail imported
only by `TrackContextMenu.slint` itself.

## Coupling / watch out
- The `height` formula (lines 74-88) reads EVERY gating property to compute
  the exact popup height (30px per visible row) — this is the trickiest part
  to keep correct after the split: it must stay in the root file (it can't
  live in either sub-component, since it needs to know the total across
  BOTH groups), and every property it references must remain declared on the
  root `TrackContextMenu` exactly as today — do not rename or move any of the
  `in property` declarations used by the height formula.
- `nav-only` mode changes semantics for almost every item (many items are
  `!root.nav-only`-gated, but go-to-album/go-to-artist/track-info are NOT
  gated on `nav-only` at all — they show in both modes) — when splitting into
  two item-group components, preserve exactly which conditions do/don't
  include `!nav-only`/`nav-only` per the current source; a mechanical
  line-for-line move (not a rewrite) is the safest way to avoid subtly
  changing which items appear in nav-only mode.
- `show-track-info` is a COMPUTED property on the root
  (`root.track-info-action && (root.nav-only || root.qobuz-actions)`) — the
  height formula and the Qobuz-items sub-component both need this exact same
  value; compute it once in the root and pass it down, don't recompute the
  formula in the sub-component (risk of drift if the formula is edited in one
  place later and not the other).

## Verify after split
- Slint compile check (`cargo build -p qbz-ui` or the project's
  slint-viewer/build-script) for `TrackContextMenu.slint` and both new files.
- Manual smoke-test: open the track-row "⋯" menu in at least 3 contexts —
  a Qobuz catalog row (qobuz-actions=true, nav-only=false), a local-file row
  (qobuz-actions=false, reveal-in-explorer-action=true), and a blacklisted/
  nav-only row — and confirm the exact same items appear/disappear as before
  the split, and the popup height matches the visible item count with no
  extra blank space or clipped rows (the height formula is the single easiest
  thing to get subtly wrong here).
- Grep for `TrackContextMenu {` importers across `ui/` to confirm every call
  site's property bindings still compile (property names/types unchanged).
