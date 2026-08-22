# crates/qbz-ui/ui/shell/SidebarNowPlayingDock.slint (149 lines)

## Summary
Large-NPB (mode 3) cover dock mounted as a root overlay by AppShell: a square album
cover flush to the sidebar's bottom-left, with two hover-revealed overlay buttons
(top-right "track info", top-left favorite heart) on top of it.

## Proposed split
Only ~19 lines over budget — the two overlay buttons are near-identical in structure
(28px rounded square, hover-reveal opacity animation, centered `QbzIcon`, `TouchArea`)
and are the cleanest extraction: a small shared `DockOverlayButton` component
parameterized by icon/tint/visibility/click, used twice.

- `shell/SidebarNowPlayingDock.slint` (~95 lines) — top-level `SidebarNowPlayingDock`
  component: the `art-size`/`height` properties, the top hairline divider, the
  `VerticalLayout` + `art` Rectangle (cover Image + placeholder glyph + `cover-ta`
  hover TouchArea), instantiating the two `DockOverlayButton`s below in place of the
  current inline Rectangles.
- `shell/sidebar-now-playing-dock/DockOverlayButton.slint` (~65 lines) — a new
  `DockOverlayButton` component: `in property <image> icon`, `in property <color>
  tint`, `in property <bool> visible-extra` (for the favorite heart's "also visible
  when active" condition — see coupling note), `in property <string>
  accessible-label`, `callback clicked()`, reproducing the 28px rounded rect +
  hover-reveal opacity + centered icon + `TouchArea` exactly as both current
  instances render it.

## Re-export surface
`shell/SidebarNowPlayingDock.slint` stays the single import surface — the
`export component SidebarNowPlayingDock` signature (`media-action` callback) is
unchanged; `DockOverlayButton` is internal-only, not exported.

## Coupling / watch out
- The two overlay buttons are NOT quite identical: the track-info button's opacity
  is `(cover-ta.has-hover || it.has-hover) ? 1.0 : 0.0` while the favorite button's is
  `(cover-ta.has-hover || ft.has-hover || QueueState.now-playing-favorite) ? 1.0 :
  0.0` (it stays visible when the track is already favorited, regardless of hover).
  The shared `DockOverlayButton` needs an `in property <bool> force-visible>` (or
  similar) that the parent binds to `false` for track-info and to
  `QueueState.now-playing-favorite` for the favorite button, OR-ed with the shared
  `cover-ta.has-hover || <own-touch-area>.has-hover` internally.
- Both buttons reference `cover-ta` (the hover TouchArea over the whole cover,
  defined in the parent `art` Rectangle) — since Slint children can't reach a
  sibling's private TouchArea state directly except through the parent, the parent
  must pass `cover-hovered: cover-ta.has-hover` down as an `in property` to each
  `DockOverlayButton` instance rather than the child referencing `cover-ta` itself.
- The track-info button is conditionally rendered at all only when
  `NowPlayingState.has-track && NowPlayingState.source == "qobuz"` (Qobuz-only, no
  info page for local/ephemeral tracks) while the favorite button only requires
  `NowPlayingState.has-track` — keep these two different `if` guards at the
  call-site level in the parent, not inside `DockOverlayButton` itself.
- The favorite button's icon/tint switch on `QueueState.now-playing-favorite`
  (filled vs outline heart, accent-color vs white tint) — these become `in
  property`s bound per-instance from the parent, not hardcoded in the shared
  component.
- `root.media-action("track", NowPlayingState.track-id, "track-info")` and
  `QueueState.toggle-now-playing-favorite()` are different actions per button — each
  instance's `clicked =>` callback in the parent stays distinct; `DockOverlayButton`
  only exposes a generic `clicked()` callback for the parent to wire.

## Verify after split
- Slint compile check on `SidebarNowPlayingDock.slint` and its importer (AppShell,
  which mounts this as a root overlay).
- Visual smoke-test: cover hover still reveals both overlay buttons; the favorite
  heart still stays visible when the track is already favorited (even without
  hover); track-info button only appears for Qobuz tracks; clicking each still
  fires the correct action (`track-info` media-action vs favorite toggle).
