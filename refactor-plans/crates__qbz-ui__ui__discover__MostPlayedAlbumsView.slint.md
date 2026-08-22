# `crates/qbz-ui/ui/discover/MostPlayedAlbumsView.slint` (198 lines)

Most Played Albums full-page listing: fixed header (nav+title left, search box right),
scrolling grid via `AlbumCollectionView`, loading/empty states, shared `ListScrollbar`.

## Proposed split

This file is only marginally over 130 lines (198) and is mostly one cohesive header +
body; a light split is enough:

- `MostPlayedAlbumsView.slint` (~110 lines) — stays the public surface: outer layout,
  `Flickable` + scroll-restore, states (loading/empty), `AlbumCollectionView` embed,
  `ListScrollbar`.
- `discover/MostPlayedSearchBox.slint` (~50 lines) — extract the inline search box
  (lines ~59-104: bordered box + `TextInput` bound to `MostPlayedAlbumsState.search` +
  placeholder text), taking no props (binds directly to the `MostPlayedAlbumsState`/
  `MostPlayedAlbumsActions` globals).

## Coupling to flag

- The search box here is a bespoke bordered `TextInput` implementation, NOT the shared
  `ExpandableSearch`/`BrowseSearch` primitive used elsewhere (e.g.
  `DiscoverBrowseView.slint` uses `BrowseSearch`) — flag this inconsistency; consider
  whether it should be unified onto `BrowseSearch` instead of extracted verbatim, but
  that's a design decision beyond a mechanical split, so the plan defaults to extracting
  as-is and leaving unification to a follow-up.
- Structure mirrors `RecentAlbumsView` (per the header comment) — check whether that view
  already has a similar header/search split to reuse the pattern from.

## Verify after split

- Slint compile check.
- Visual smoke test: search filters the grid, loading spinner, empty state, scrollbar.
