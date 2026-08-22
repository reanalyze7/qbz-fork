# `crates/qbz-ui/ui/discover/DiscoverBrowseView.slint` (237 lines)

Discover "View all" full-list page: fixed header (nav + title left, search/genre/view-mode
tools right), infinite-scroll grid/list body, genre-filter popup overlay.

## Proposed split

- `DiscoverBrowseView.slint` (~110 lines) — stays the public surface: `export component
  DiscoverBrowseView`, the outer layout, the `Flickable` with its infinite-scroll
  `changed viewport-y` handler (load-more logic — keep this with the main component since
  it directly drives `DiscoverBrowseActions.load-more()`), and composes the header below.
- `discover/DiscoverBrowseHeader.slint` (~60 lines) — new component wrapping the fixed
  56px header (nav buttons + title left cluster, search/genre/view-mode right cluster,
  lines ~44-103), taking title as a property and using `DiscoverBrowseState`/
  `GenreFilterState`/`GenreFilterActions` globals directly (already globals, no
  prop-threading needed).
- Genre-filter overlay (backdrop + `GenreFilterPopup`, lines ~221-236) can either stay in
  the parent (it's small and coupled to `root.width`) or move into the header component —
  recommend keeping it in the parent since it's positioned relative to `root.width`.

## Coupling to flag

- Nearly identical header/structure to `MostPlayedAlbumsView.slint` (same batch) and
  likely other Discover "view all" pages — consider whether a shared
  `discover/BrowseHeader.slint` could serve both instead of two near-duplicate headers.
  Note this cross-reference if that file's plan is written separately.
- The infinite-scroll threshold (600px) and `content-offset: 8px` windowing offset are
  tightly coupled to the exact header height (56px) — if the header is extracted, keep
  these values in sync and comment why.

## Verify after split

- Slint compile check.
- Visual smoke test: infinite scroll load-more, search/genre filter, grid/list toggle,
  scroll-position restore on back-nav.
