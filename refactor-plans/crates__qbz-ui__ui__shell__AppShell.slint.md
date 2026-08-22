# crates/qbz-ui/ui/shell/AppShell.slint (932 lines)

## Summary
The post-login root shell: header + sidebar + content region (view-dispatch over
~25 possible `ContentView`s) + player bar, plus the app-wide dynamic background
layer, swipe/back-forward gesture handling, the unified track context menu, and
every global overlay/modal (~30 of them) mounted at the root for z-order control.

## Proposed split
Slint components can't be split mid-body, but `AppShell` is really five
composable regions that can each become their own small wrapper component,
recombined by a slimmed-down top-level `AppShell.slint`. This mirrors the
`JumpNavBar.slint` precedent (extract sub-widgets, compose back).

- `shell/AppShell.slint` (~120 lines) — top-level `AppShell` component: the
  `import`s, the shell-level callbacks, the `qobuz-view-blocked` property, the
  `background`, and composition of the extracted pieces below in place of their
  current inline blocks. Keeps the outermost `Rectangle` + `VerticalLayout`
  scaffold (header / content-row / player-bar) since that's the shell's own
  layout, not extractable content.
- `shell/appshell/GestureLayer.slint` (~90 lines) — the bottom-most `TouchArea`
  (currently lines 218-293): mouse back/forward buttons + two-finger swipe
  accumulation for history navigation. Self-contained: only reads/writes
  `NavState`. New component `AppShellGestureLayer`, no props needed (reads
  `NavState` globally), mounted first (z-order preserved).
- `shell/appshell/DynamicBackground.slint` (~50 lines) — the two `if
  ShellState.app-background-active...` Image/ImmersiveAtmosphere/scrim blocks
  (currently lines 187-216). New component `AppShellDynamicBackground`, no props
  (reads `ShellState`/`AppearanceState`/`ImmersiveState`/`NowPlayingState`
  globally), needs `width`/`height` bound to the parent (pass `root.width`/
  `root.height` explicitly since it's edge-pinned/absolute).
- `shell/appshell/ContentRegion.slint` (~420 lines, the biggest remaining chunk)
  — the entire content-view dispatch: all the `if NavState.view == ...` arms
  (lines 382-605) INSIDE the bordered/clipped content Rectangle. New component
  `AppShellContentRegion` forwarding `open-album`/`open-artist`/`media-action`/
  `discover-view-all`/`settings-*` callbacks straight through to `root.*` (same
  signatures AppShell already exposes), so the parent's callback wiring is
  unchanged. This alone is still oversized — split further:
  - `appshell/content/BrowseViews.slint` (~120 lines) — home, discover-browse,
    playlist-browse, recent-albums, most-played-albums arms.
  - `appshell/content/DetailViews.slint` (~150 lines) — album, local-album,
    artist, artist-releases, musician, label, label-releases, location arms.
  - `appshell/content/LibraryViews.slint` (~100 lines) — favorites,
    playlist-manager, mixtapes, collections, mixtape-detail, offline-manager,
    blacklist-manager, local-library, mix, playlist arms.
  - `appshell/content/SettingsSearchViews.slint` (~60 lines) — settings, search
    arms (these two have the widest per-arm callback surfaces —
    settings-bool/select/slider/string/reset/release-device).
  Each sub-file needs the `OfflinePlaceholder`/offline-gating `if` duplicated or
  hoisted to the parent `ContentRegion.slint` (the placeholder decision is
  view-independent, so keep it in `ContentRegion.slint` itself, only the
  per-view arms move out).
- `shell/appshell/GlobalOverlays.slint` (~230 lines) — every modal mounted after
  the outer `VerticalLayout` (HeaderMenuOverlay, Cortinilla, SidebarFolderPopup,
  SidebarTooltip, SidebarPlaylistsPopup, PlaylistAddModal,
  PlaylistDuplicateConfirmModal, EditPlaylistModal, CreateFolderModal,
  CreateMyQbzModal, MyQbzEditModal, MyQbzMixModal, AddToMixtapeModal,
  EphemeralPlayChoiceModal, FolderEditModal, LibFolderEditModal, TagEditorModal,
  PlaylistImportModal, DacWizardModal, TrackInfoModal, AlbumCreditsModal,
  AlbumBookletModal, LogViewerModal, SettingsExportModal, ReportIssueModal,
  DiscoverConfigModal, KeyboardShortcutsModal, CustomizeShortcutsModal,
  LinkResolverModal, AboutModal, WhatsNewModal, DragGhost, Toast,
  TooltipOverlay, ArtPreviewOverlay). New component `AppShellGlobalOverlays`,
  no props (every modal self-gates on its own State.open) EXCEPT
  `header-menu-navigate` which forwards through. CRITICAL: declaration order
  inside this component must be preserved EXACTLY (ADR-009: z-order = decl
  order) — do not alphabetize or "clean up" the ordering when moving.
- `shell/appshell/TrackMenu.slint` (~90 lines) — the "Unified track context
  menu" section (lines 856-931): `track-menu-qobuz-source`/
  `track-menu-open-watch`/`track-menu-view-watch` properties + the
  `unified-track-menu := TrackContextMenu` instance. New component
  `AppShellTrackMenu`, forwarding `media-action` through (needs `NavState.view`
  read directly — no prop needed, global state).

## Re-export surface
`shell/AppShell.slint` stays the ONE import surface — `app.slint`'s
`import { AppShell } from "shell/AppShell.slint";` is untouched. All the
extracted sub-components (`GestureLayer`, `DynamicBackground`, `ContentRegion`
and its own sub-splits, `GlobalOverlays`, `TrackMenu`) are imported and composed
only inside `AppShell.slint` itself — none are exported/imported elsewhere,
so no other file's imports change.

## Coupling / watch out
- Z-ORDER IS DECLARATION ORDER (ADR-009/010, called out repeatedly in this
  file's own comments) — the split MUST preserve the exact current stacking:
  GestureLayer -> DynamicBackground -> main VerticalLayout (header/content/
  queue/player) -> Large-NPB dock -> HeaderMenuOverlay -> Cortinilla ->
  SidebarFolderPopup -> ... -> Toast -> TooltipOverlay -> ArtPreviewOverlay ->
  TrackMenu. When these become separate components composed in
  `AppShell.slint`, keep them instantiated in this SAME order or the whole
  stacking model breaks silently (no compile error, just wrong z-order at
  runtime).
- `qobuz-view-blocked` (property on root, computed from `OfflineState`/
  `NavState`) is read by the extracted `ContentRegion` — must be passed in as
  an `in property` (or kept in `AppShell.slint` root and the region reads
  `root.qobuz-view-blocked` via a parent-scope reference, which Slint does NOT
  support across component boundaries) — so this property MUST be threaded
  down as an explicit `in property <bool>` to `ContentRegion`.
- The content-height arithmetic (`root.height - Layout.header-height -
  Layout.player-bar-height - 8px`) references `root.height` (the AppShell's own
  height) — if `ContentRegion` becomes a separate component, it needs its OWN
  `height` passed in or bound the same way; don't let it silently read a
  different `root`.
- `track-menu-view-watch`/`track-menu-open-watch` `changed` handlers call
  `unified-track-menu.show()`/`.close()` by LOCAL element name — these must
  stay inside the same component as the `TrackContextMenu` instance
  (`TrackMenu.slint`), can't be split further.
- The reduce-motion `Timer` (coarse-tick-ms) and the viz-should-run `changed`
  handler are shell-level cross-cutting concerns reading multiple globals
  (ShellState/NowPlayingState/AppearanceState/VisualizerState) — keep these in
  the top-level `AppShell.slint` itself, not in any extracted sub-component,
  since they don't belong to a single visual region.
- Every extracted content-view arm forwards `open-album`/`open-artist`/
  `media-action` etc. straight to `root.*` — when nested two levels deep
  (AppShell -> ContentRegion -> BrowseViews/DetailViews/...), each intermediate
  component needs to re-declare and re-forward these callbacks (Slint callback
  forwarding is not transitive through unrelated component boundaries).

## Verify after split
- Slint compile check (`cargo build` triggers the slint build script, or
  `slint-viewer` directly) on `AppShell.slint` and `app.slint` (its sole
  importer).
- Manual/visual smoke-test covering: header, sidebar (mini + expanded), every
  major content view (home, album, artist, favorites, settings, search, local
  library, playlist), the Queue side panel, the Large NPB dock, EVERY modal
  opening/closing (spot-check 5-6, not all 30), the track context menu from
  multiple surfaces, swipe-to-navigate (if a touchpad/trackpad is available),
  the dynamic background modes (Ambient + Blurred art), and z-order (open two
  overlapping overlays, confirm the expected one is on top).
- Confirm no other `.slint` file imports any of the newly-created
  `appshell/*.slint` sub-components directly (`grep -rn "appshell/" crates/qbz-ui/ui`).
