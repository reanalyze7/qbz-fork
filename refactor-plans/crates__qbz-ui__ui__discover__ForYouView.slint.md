# crates/qbz-ui/ui/discover/ForYouView.slint (206 lines)

## Summary
The Discover > For You tab layout: a skeleton-loading gate followed by a
single `for desc in DiscoverState.foryou-sections` repeater whose body is an
if-chain that mounts the right heterogeneous row component (album/track
carousel, artist carousel, pinned carousel, Qobuz Mixes tiles, Spotlight)
per prefs-driven section descriptor id, each with its own empty-data
self-hide gate.

## Proposed split
The natural cut is the repeater's per-iteration delegate — it's one long
if-chain that can become its own component, leaving `ForYouView.slint` as a
thin loop + skeleton gate.

- `ForYouView.slint` (~50 lines) — imports, the `show-skeleton` property
  and its gating logic, the `if root.show-skeleton: HomeSkeleton {}` line,
  and the `for desc in DiscoverState.foryou-sections: ForYouSectionRow { ... }`
  loop that forwards all six callbacks (`open-album`, `open-artist`,
  `media-action`, `play-track`, `play-top-tracks`, `open-mix`) plus `desc`
  as an `in property` to the delegate.
- `discover/ForYouSectionRow.slint` (~165 lines) — the extracted delegate
  component: `in property <SectionDescriptor> desc` (or whatever the loop
  variable's type is) plus the same six callbacks, containing the entire
  if-chain (`qobuzMixes`, `pinned`, `releaseWatch`, `continueListening`,
  `recentlyPlayedAlbums`, `topArtists`, `favoriteAlbums`,
  `mostPlayedAlbums`, `similarAlbums`, `rediscoverLibrary`,
  `artistsToFollow`, `artistSpotlight`) unchanged, each arm's callback
  forwarding to the delegate's own callbacks (which `ForYouView.slint`
  re-forwards).
  If 165 lines is still judged too close to budget after the real split,
  this file can be further divided by extracting the artist-carousel arms
  (`topArtists`, `artistsToFollow`) or the "self-contained, no user callback
  needed" `qobuzMixes`/`artistSpotlight` arms into a second delegate — but
  a single `ForYouSectionRow.slint` should already land near budget since
  it's mostly repeated small `if` blocks.

## Re-export surface
`ForYouView.slint` remains the single import surface
(`import { ForYouView } from "discover/ForYouView.slint";`) — its root
component name and the six callback signatures are unchanged; the new
`ForYouSectionRow` is an internal implementation detail imported only by
`ForYouView.slint` itself.

## Coupling / watch out
- Confirm the loop variable's actual type (need to check `DiscoverState`'s
  `foryou-sections` property type in `state.slint` — likely a struct with an
  `id: string` field) so `ForYouSectionRow`'s `desc` property is typed
  correctly, not just `string`.
- Each arm reads its OWN data field directly off a *different* global
  (`ForYouState.release-watch`, `ForYouState.recent-tracks`,
  `PinnedState.items`, `DiscoverState` isn't touched inside arms) — the
  delegate component must import all of `ForYouState`, `PinnedState`, and
  `HomeActions` (used by `mostPlayedAlbums`'s `view-all-clicked`), matching
  the current top-of-file imports.
- The comment block at the top explains WHY `essentialsByGenre` has no arm
  (Slice-2c-blocked) — carry that comment into `ForYouSectionRow.slint`
  since it documents behavior of the if-chain, not the loop.
- `HomeActions.open-most-played-albums()` is called directly from inside the
  `mostPlayedAlbums` arm (not via a callback) — this direct-global-call
  pattern must be preserved exactly when the arm moves files.

## Verify after split
- Slint compile check (`slint-viewer` or the crate's build script) on both
  files.
- `cargo build -p qbz-ui`.
- Visually smoke-test the Discover > For You tab: confirm skeleton shows
  while loading, then every enabled section renders in the prefs-configured
  order with working carousel clicks (open-album/open-artist/play-track) and
  Spotlight/Qobuz-Mixes navigation.
