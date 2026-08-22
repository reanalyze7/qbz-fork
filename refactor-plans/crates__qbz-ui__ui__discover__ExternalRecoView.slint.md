# crates/qbz-ui/ui/discover/ExternalRecoView.slint (211 lines)

## Summary
Discover > Recommendations (4th) tab view: 9 self-hiding carousel rows
(2 artist rails, 4 album rows, 2 track "slim" rows, 1 top-artists row) each
gated on its own `length > 0` / `pending-*` flag from `ExternalRecoState`,
plus an initial full-page skeleton and an empty state.

## Proposed split
The 9 row blocks are near-identical boilerplate (content-or-skeleton
conditional), differing only in which state field/carousel-component/title
they use. Two viable approaches — given this is a single `export component`
with computed properties feeding every row, prefer extracting a couple of
small named sub-components rather than fragmenting the flow itself:

- `ExternalRecoView.slint` (~120 lines) — KEEP as the main file: imports,
  `export component ExternalRecoView`, the `any-content`/`any-pending`
  computed properties (lines 26-48), the initial full-page skeleton gate,
  and the empty-state block (lines 196-210). The 9 row blocks become calls
  to the two new wrapper components below instead of repeating the
  `if X.length > 0: Carousel {...} if X.length == 0 && pending: Skeleton
  {...}` pattern inline.
- `discover/RecoAlbumRow.slint` (~55 lines) — a small wrapper component
  taking `section: DiscoverSection`, `pending: bool`, forwarding
  `album-clicked`/`media-action`, that internally renders
  `if section.albums.length > 0: Carousel {...} if ... && pending:
  SkeletonCarouselRow {...}` — replaces the 4 near-identical album blocks
  (rec-albums, fresh-releases, deep-cut-albums, top-albums; lines 92-105,
  108-120, 150-163, 165-178).
- `discover/RecoArtistRow.slint` (~45 lines) — same wrapper shape for the
  2 artist-rail blocks + the top-artists block (lines 60-90, 180-194),
  taking `title`, `artists`, `pending`, forwarding `artist-clicked`/
  `media-action`.
- `discover/RecoTrackRow.slint` (~40 lines) — same wrapper shape for the
  2 Weekly track rows (lines 123-148), taking `title`, `items`, `section-
  id` (for the `media-action("ext-reco-list", section-id, action)` call),
  `pending`, forwarding `play-track`.

If the reviewer doing the actual split prefers NOT introducing new wrapper
components (to avoid an extra indirection layer for what is fundamentally
one view), an acceptable fallback is to leave `ExternalRecoView.slint` as
one file and instead extract only the `any-content`/`any-pending` boolean
+ empty-state block into a tiny `discover/RecoEmptyState.slint`, accepting
the main file staying near ~180 lines — but that does NOT hit the 130-line
target, so the wrapper-component approach above is the one that actually
satisfies the rule.

## Re-export surface
`ExternalRecoView.slint`'s `export component ExternalRecoView` stays the
only import surface (imported from the Discover tab container, e.g.
`DiscoverView.slint`). The three new row-wrapper components are internal to
`discover/` and imported only by `ExternalRecoView.slint` itself — they are
not part of the public API other files need to know about.

## Coupling / watch out
- `ExternalRecoState` (from `../state.slint`) is read directly in the row
  conditions today; when wrapping in `RecoAlbumRow`/`RecoArtistRow`/
  `RecoTrackRow`, pass the relevant slice (`section`/`artists`/`items` +
  `pending`) as `in property` rather than having the wrapper import
  `ExternalRecoState` itself — keeps the wrapper reusable and testable in
  isolation, and keeps `any-content`/`any-pending` (which need to see ALL 9
  fields) in the main file where they already are.
- The Weekly track rows' `list-action` callback hardcodes the section id
  string (`"weekly-exploration"` / `"weekly-jams"`) into the `media-action`
  call (lines 128, 141) — when wrapping into `RecoTrackRow`, this id must
  become a parameter, not a hardcoded literal in the wrapper, since the
  wrapper is instantiated twice with different ids.
- `card-height`/`card-size` differ slightly between album rows (266px
  content / 180px skeleton) and artist rows (256px content / 180px
  skeleton) — preserve these per-row constants exactly when parameterizing
  the wrappers.

## Verify after split
- Slint compile check.
- Manually smoke-test the Discover > Recommendations tab in every state:
  loading (full skeleton), progressive fill (per-row skeletons swapping to
  content), fully loaded, and empty (no Last.fm/ListenBrainz connected, no
  history).
