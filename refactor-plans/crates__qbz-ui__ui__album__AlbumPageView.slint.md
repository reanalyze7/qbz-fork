# crates/qbz-ui/ui/album/AlbumPageView.slint (1277 lines)

## Summary
The Album detail page: back/nav, a header (artwork + credits + meta +
description + action-button row + cover context menu), a toolbar (quality
badge, search, multi-select, Hi-Res filter), the track list (disc/work
headers), a label/external-links sidebar, bottom "More from artist" /
"Suggestions" carousels, and a full-description modal.

## Proposed split
By-domain component extraction (Slint has no pure/IO/render distinction —
this is UI-only) into sibling files under `crates/qbz-ui/ui/album/`, each
exporting one component that `AlbumPageView.slint` imports and composes.
This mirrors how `TrackRow`, `AlbumContextMenu`, `DiscHeaderMenu` etc. are
already extracted into `primitives/`.

- `AlbumPageView.slint` (~115 lines) — becomes the thin composition root:
  imports, `export component AlbumPageView` with only its own
  properties/callbacks (`hires-only`, `open-artist`, `media-action`,
  `show-description`, `desc-display`, `header-light`/`header-atmo-on`/
  `hdr-*` color properties, `atmo-height`), the outer `Flickable` +
  `page := VerticalLayout` skeleton delegating each section to the new
  components below, plus the `ListScrollbar` and full-description modal
  (kept inline since it's small and page-specific — see below).
- `album/SidebarCard.slint` (~70 lines) — the `SidebarCard` component
  (lines 33-97, currently a private in-file component) — export it.
- `album/BrandLink.slint` (~40 lines) — the `BrandLink` component (lines
  106-145) + the trivial `SidebarHeading` (lines 99-104, small enough to
  fold into this file or `SidebarCard.slint`'s file as a second export).
- `album/AlbumHeader.slint` (~330 lines) — everything currently inside the
  "Album header" `HorizontalLayout` (lines 282-633): artwork + cover
  context menu, title/credits/meta/description block, and the action
  button row (play/shuffle/edit/favorite/booklet/mixtape/info/⋯ menu).
  Takes `AlbumState`/`ArtworkActions` globals directly (already global
  singletons, no prop-drilling needed) plus the `hdr-*` color properties
  and `media-action`/`open-artist` callbacks forwarded from the root.
- `album/AlbumToolbar.slint` (~140 lines) — the toolbar `HorizontalLayout`
  (lines 677-821: quality badge, search box, multi-select toggle, Hi-Res
  filter toggle) plus the `MultiSelectBar` usage (lines 823-839) that
  logically belongs with the toolbar's multi-select toggle. Owns the
  `hires-only` property (or takes it as an `in-out property` bound from
  the root, since the root's `for track in ...` filter reads it — see
  coupling note below).
- `album/AlbumTrackList.slint` (~230 lines) — the column header (lines
  842-913) + the `for track[index] in AlbumState.tracks` loop (lines
  915-1039: disc header, work header, `TrackRow`). Takes `hires-only` as
  an `in property` for the visibility filter.
- `album/AlbumSidebar.slint` (~95 lines) — the label/external-links
  sidebar `VerticalLayout` (lines 1042-1133), composed from `SidebarCard`
  and `BrandLink`.
- `album/AlbumDescriptionModal.slint` (~80 lines) — the full-description
  modal (lines 1202-1276), taking `show-description` as an `in-out
  property` (or a callback pair `close()`/visibility bound from the root)
  and `AlbumState.description` directly from the global.

## Re-export surface
`AlbumPageView.slint` stays the single import surface — every other
`.slint` file that does `import { AlbumPageView } from "./album/AlbumPageView.slint";`
(the shell's page router) is unaffected; only this file's *internal*
composition changes, plus new sibling imports at its own top (`import {
AlbumHeader } from "./AlbumHeader.slint";` etc., relative to the same
`album/` directory).

## Tricky coupling / watch out
- **`hires-only` spans 3 sections**: declared on `root` (line 150), read by
  the toolbar's filter-toggle button (line 788-818) AND by the track-list
  loop's `visible:` binding (line 933). If `AlbumToolbar` and
  `AlbumTrackList` become separate components, `hires-only` must either (a)
  stay a property on the `AlbumPageView` root and be passed into both
  children as an `in-out property` two-way-bound (`hires-only <=>
  toolbar.hires-only` is NOT how Slint two-way binds into a child — use
  `in-out property` + explicit binding `toolbar.hires-only: root.hires-only;
  toolbar.hires-only-changed => root.hires-only = ...` or simpler: keep
  `hires-only` on the root and have both `AlbumToolbar` (as `in-out
  property`) and `AlbumTrackList` (as `in property`) reference it via
  `<=>` alias syntax, which Slint DOES support for property aliasing
  across component boundaries), or (b) simplest: keep the Hi-Res toggle
  button itself inline in `AlbumPageView.slint` (not extracted into
  `AlbumToolbar`) so `hires-only` never needs to cross a component
  boundary — recommend this simpler option given the property's use-span.
  **This was called out explicitly in the task brief as recently-touched
  and must stay a cohesive, working block — do not split the toggle button
  from the property it mutates.**
- **`atmo-height` / `header-atmo-on` / `header-light`** properties are
  computed on the root from `AppearanceState`/`ShellState` globals and
  consumed by 3 background `Rectangle`/`ImmersiveAtmosphere` elements that
  sit OUTSIDE `page := VerticalLayout` (siblings inside `flick :=
  Flickable`, lines 226-260) while `atmo-height` itself depends on
  `header-divider.y` INSIDE `page` (line 192: `page.y + header-divider.y`).
  If `AlbumHeader.slint` is extracted, `header-divider` (line 638, the
  divider marking the end of the header) must remain accessible to the
  root's `atmo-height` binding — either keep `header-divider` in the root
  file (NOT inside `AlbumHeader.slint`) as a thin separator the root places
  right after `<AlbumHeader/>`, or have `AlbumHeader` expose its own height
  as an output property (`out property <length> content-height`) that the
  root reads instead of reaching into a child's named element (Slint does
  not allow reaching into a child component's internal named elements from
  outside, so `header-divider.y` MUST move to the root or be exposed via an
  output property — this is the single trickiest coupling point in this
  split).
- **`AlbumState`/`ArtworkActions`/`NowPlayingState`/`AppearanceState`/
  `ShellState`/`TooltipState`/`DragState`/`NavState`/`UiFocusState`
  globals** are all Slint singletons already globally accessible — no
  prop-drilling needed for state reads; only the root-owned properties
  (`hires-only`, `show-description`, `desc-display`, `hdr-*`,
  `header-atmo-on`, `atmo-height`) need explicit passing into extracted
  children.
- **Scroll-restore logic** (`sr-armed`, `sr-restore()`, the `init =>` /
  `changed viewport-y =>` handlers on `flick`, lines 211-222) must stay on
  the `Flickable` in the root file — do not extract this into a child
  component, it is tightly bound to `flick`'s own `viewport-height`/
  `viewport-y`.
- **The gold Hi-Res badge / color logic** lives in `AudioStamp.slint`, not
  in this file — `AlbumPageView.slint`'s own Hi-Res-related bit is only the
  filter TOGGLE (`root.hires-only`), a separate and unrelated concern from
  the badge; no cross-file coupling between the two files despite both
  mentioning "Hi-Res".

## Verify after split
- `cargo build -p qbz-ui` (or wherever the Slint build script lives) to
  confirm the `.slint` files still compile through `slint-build`/
  `slint-compiler`.
- Visually smoke-test: open an album page, confirm header renders
  (artwork, credits, action buttons, cover context menu), toolbar search +
  multi-select + Hi-Res filter still work, track list still groups by
  disc/work, sidebar shows label/external links, bottom carousels render,
  and the full-description modal opens/closes — since this is a large
  visual page, a manual click-through (or the `run` skill if available) is
  the real regression check, not just "it compiles."
- Confirm the Hi-Res filter toggle (owner-requested, this session) still
  hides non-hires tracks live without a network round trip, and the
  gold-badge styling in `AudioStamp.slint` (a different file) is untouched.
