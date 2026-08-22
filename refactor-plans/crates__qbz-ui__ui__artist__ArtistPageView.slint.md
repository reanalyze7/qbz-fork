# crates/qbz-ui/ui/artist/ArtistPageView.slint (1830 lines)

## Summary
The full Artist detail page: gradient/atmosphere header (portrait, name, bio,
follow/network/overflow actions), sticky JUMP TO bar, Popular Tracks +
discography release sections + Appears On + Playlists + "Other" (collapsed),
a library-tab variant, a right-side Network/Magazine sidebar overlay, and a
full-biography modal — all in one component tree, by far the largest file in
the repo.

## Proposed split
This is a single `export component ArtistPageView` with deeply-nested inline
blocks; Slint has no free functions to extract to, so the split must be into
sibling `.slint` files each exporting a sub-component that `ArtistPageView`
composes. Split by "page region" (the natural seams already marked by `// ---`
comment banners in the file):

- `artist/ArtistPageView.slint` (~110 lines) — kept as the export surface: top
  imports, the `component SectionTitle`/`SidebarLink` micro-helpers (or moved to
  `artist/ArtistHelpers.slint`, ~50 lines, see below), the `export component
  ArtistPageView` shell itself: state properties (`show-bio`, `search-visible`,
  `active-jump-tab`, `header-light`/`header-atmo-on`/color properties,
  `net-cramped` watcher), the outer `Flickable` + `page` `VerticalLayout`
  scaffold, and composition of the extracted sub-components below via their
  callbacks/properties.
- `artist/ArtistHelpers.slint` (~60 lines) — `SectionTitle` and `SidebarLink`
  (lines 45-98), used by both the header and the sidebar; exported so other
  new files can import them.
- `artist/ArtistHeader.slint` (~330 lines, lines ~211-611) — the atmosphere
  background rectangles, the 200px portrait + artwork context menu, name +
  bio + read-more, the action-button row (Follow/Network/Overflow menu +
  catalog/library SegmentedTabBar), and the hidden-artist warning banner.
  Takes `header-light`/`hdr-strong`/`hdr-body`/`hdr-on-surface`/`atmo-height`
  as `in` properties (computed in the root, since they also gate the atmo
  overlay drawn by the root Flickable) plus a `media-action` callback and
  `show-bio` in-out property.
- `artist/PopularTracksSection.slint` (~230 lines, lines ~662-828) — Popular
  Tracks header + play/select/overflow menu + bulk MultiSelectBar + the
  `for track in top-tracks` TrackRow loop + Load more/View less. Owns its own
  `top-tracks-expanded`/`top-tracks-preview` local state (or receives them as
  `in-out property` if the root still needs `top-tracks-preview` elsewhere —
  it currently doesn't, so localize it here).
- `artist/DiscographySection.slint` (~220 lines, lines ~830-1005) — latest-
  release highlight card, `for section in release-sections` (excluding
  "other") → ReleaseGrid, Appears On block (with its own preview/expand
  state), Playlists carousel, and the "Other" collapsed block with its own
  `other-expanded` state.
- `artist/LibraryTabSection.slint` (~40 lines, lines ~1009-1038) — the
  `artist-tab == "library"` branch (library tracks + AlbumCollectionView).
- `artist/NetworkSidebar.slint` (~330 lines, lines ~1122-1674, minus the
  magazine sub-block if split further) — the whole right-edge overlay: tab
  header (Network/Magazine + close), the Network tab body (Origin, Labels,
  Similar Artists, Relationships, Discovery-dismiss rows) and the Magazine
  tab body (stories list). This one alone is close to 350 lines; if it stays
  over 130 after drafting, further split into `NetworkSidebar.slint` (header +
  tab shell, ~90) + `NetworkSidebarNetworkTab.slint` (~190) +
  `NetworkSidebarMagazineTab.slint` (~60) — do this split when implementing,
  guided by actual line counts.
- `artist/BiographyModal.slint` (~85 lines, lines ~1745-1829) — the
  full-bio scrim + panel + Flickable body, taking `show-bio` as an `in-out
  property` (or a callback pair `close()`/visible bound from root).

## Re-export surface
`artist/ArtistPageView.slint` stays the only file other `.slint` files
import `ArtistPageView` from (its `export component` name and file path are
unchanged — only its *internal* body shrinks by importing the new sibling
components). None of the new files need their own "mod.rs" equivalent since
Slint has no re-export indirection; callers of `ArtistPageView` (e.g. the
shell/router) are completely unaffected as long as the top-level export stays
in this file with the same name and public callbacks (`open-album`,
`media-action`).

## Coupling / watch out
- Heavy cross-region property reads: `body-row.absolute-position.y` and
  `body-row.y` are referenced by the JUMP TO bar's `natural-top` AND the
  Network sidebar's `natural-top`, AND `root.atmo-height` (`page.y +
  body-row.y`) — `body-row` must stay a named element visible to whichever
  file ends up containing the JUMP TO bar and sidebar (likely still the root
  shell, since both are siblings of the Flickable, not children of it). Keep
  the JUMP TO bar (`jump-nav`) and `ListScrollbar`/`sidebar-panel` block in
  the root `ArtistPageView.slint` rather than extracting them, OR pass
  `body-row-y: length` as an `in` property from root down into a
  `NetworkSidebar`/`JumpBar` component computed once in root.
  - Similarly `page-flickable.viewport-y`/`page-flickable.absolute-position`
    are read by the scrollbar, jump bar, and sidebar — all three should
    probably stay as root-level siblings (not moved into a sub-file) even
    though they're visually "at the end" of the file; only the deeply nested
    BODY content (header, popular tracks, discography, library tab, sidebar
    body) should move to sibling files.
- `ArtistState` (imported from `../state.slint`) is read from nearly every
  extracted region — no special handling needed since Slint globals are
  visible from any file that imports them, just re-import `ArtistState`/
  `ArtistActions`/etc. in each new file.
- `root.media-action(...)` and `root.open-album(...)` callbacks are invoked
  from deep inside almost every region — each extracted component needs its
  own `callback media-action(string,string,string)` / `callback
  open-album(string)` forwarded up to the root's callbacks of the same name
  (Slint has no implicit "call parent's callback" — must thread explicitly).
- `NetworkSidebarState.open`/`.active-tab` toggling interacts with
  `net-cramped`/`changed net-nav-watch` in the root — keep those `changed`
  handlers in the root file since they mutate global state on artist
  navigation, not view-local state.

## Verify after split
- `cargo build -p qbz-ui` (or the workspace's slint-viewer/build-script check)
  to confirm all extracted `.slint` files parse and the imports resolve.
- Visually smoke-test: open an artist page, confirm header/atmosphere,
  Popular Tracks multi-select + Load more, discography sections + Other
  toggle, Appears On, the Network/Magazine sidebar toggle + sticky behavior on
  scroll, and the full-bio modal all still render and their callbacks
  (play/follow/pin/blacklist/share/open-album) still route through to
  `root.media-action`/`root.open-album` unchanged.
- Check every other `.slint` file that imports `ArtistPageView` (likely the
  shell/router) still compiles unchanged.
