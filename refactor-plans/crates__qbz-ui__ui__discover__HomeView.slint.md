# crates/qbz-ui/ui/discover/HomeView.slint (652 lines)

## Summary
The Discover "Home" tab: a fixed 56px toolbar (Home/Editor's Picks/For You/
Recommendations tab pill + genre filter + settings gear) over a scrollable
column of section carousels (new releases, playlists, recently played,
popular, library albums, most played, mixes, release watch, top artists,
pinned), driven by a prefs-ordered descriptor list from the Rust side.

## Proposed split
Slint components can live in their own files and be `import`ed, so split by
extracting the small standalone components first, then the giant
`home-content` repeater body into a couple of section-group components:

- `HomeView.slint` (~150 lines) — becomes the re-export/entry surface: the
  `export component HomeView` root, its toolbar (`left-controls`/
  `right-controls`/`tab-pill`), the Flickable/ListScrollbar scroll host
  scaffold, and the genre-filter popup overlay at the bottom. It `import`s
  the extracted components below and keeps the single `for desc in (...)`
  repeater, but each `if desc.id == ...` arm's BODY is delegated to a
  small wrapper component instead of being inlined.
- `home/HomeToolbarTab.slint` (~30 lines) — the `Tab` component (tab-pill
  button).
- `home/HomeGenreButton.slint` (~50 lines) — the `GenreButton` component.
- `home/HomeGearButton.slint` (~25 lines) — the `GearButton` component.
- `home/RecentPlaceholder.slint` (~20 lines) — the `RecentPlaceholder`
  component (shared by the two "no data yet" placeholders).
- `home/HomeAlbumSections.slint` (~120 lines) — the generic album-carousel
  arm (`newReleases`/`pressAwards`/`idealDiscography`/`editorPicks`/
  `qobuzissimes`/`mostStreamed`-as-carousel) plus `favoriteAlbums`,
  `mostPlayedAlbums`, `releaseWatch` — these are all "plain `Carousel` bound
  to `desc.section` or a `HomeState.*` field" arms, cohesive as one
  "album carousels" component that takes `desc` in and re-emits
  `album-clicked`/`media-action`/`view-all-clicked` (or the specific
  `HomeActions.*` callback each arm uses) to its parent.
- `home/HomePlaylistsSection.slint` (~70 lines) — the `qobuzPlaylists` arm
  (title + `PlaylistTagFilter` + `PlaylistCarousel` + empty-state text).
- `home/HomeRecentSections.slint` (~70 lines) — `recentlyPlayedAlbums` and
  `continueListening` arms (carousel/slim-carousel + their placeholders).
- `home/HomeMiscSections.slint` (~60 lines) — `mostStreamed`-as-slimGrid,
  `qobuzMixes`, `topArtists`, `pinned` arms (the remaining one-off
  descriptor ids).

Each extracted "Sections" component takes `in property <...> desc` (the
descriptor) plus whatever `HomeState`/`PinnedState`/`DiscoverState` fields
it needs (Slint globals are accessible from any file via `import`, so no
extra plumbing needed there), and re-declares the same callbacks
(`album-clicked`, `open-artist`, `media-action`, `discover-view-all`) that
bubble up to `HomeView`'s root via `root.album-clicked(id)` etc.

## Re-export surface
`HomeView.slint`'s `export component HomeView` stays the only thing anyone
outside `discover/` imports (e.g. `AppShell.slint` or wherever Home is
mounted via `import { HomeView } from "discover/HomeView.slint";`) — that
import line does not change. The new `home/*.slint` files are internal to
`HomeView.slint` and never imported directly by outside callers.

## Coupling / watch out
- Every extracted section component needs its own callback re-declarations
  and the parent `for desc in (...)` loop must wire each callback through
  (e.g. `HomeAlbumSections { desc: desc; album-clicked(id) => {
  root.album-clicked(id); } ... }`) — mechanical but easy to get an id
  wrong; test each arm's data-gating condition (`HomeState.X.length > 0`)
  survives the move verbatim.
- The single `for desc in (...)` repeater currently contains ALL the
  `if desc.id == ...` arms as siblings inside one `VerticalLayout` per
  `desc` — splitting them into separate components means each new
  component itself needs to be an `if desc.id == "..."：Component { }` arm
  inside the *same* per-desc `VerticalLayout` in `HomeView.slint`, not a
  nested wrapper, so spacing/layout behavior (`spacing: 0px` on the outer,
  `alignment: start`) is unchanged.
- `GenreFilterState`/`PinnedState`/`HomeState`/`DiscoverState`/
  `ShellState`/`AppearanceState`/`SettingsState`/`TooltipState`/`NavState`
  are all Slint globals imported once at the top of `HomeView.slint` today
  — each new file needs its own `import { ... } from "../state.slint";`
  line for just the globals it touches.
- The scroll-restore logic (`sr-armed`/`sr-restore`, tied to
  `NavState.restore-scope == "home"`) lives on the `Flickable` in the
  toolbar/scaffold section — keep it in `HomeView.slint` itself, it's not
  part of any individual section.

## Verify after split
- Build the Slint UI (`cargo build -p qbz-ui` or the project's slint-viewer/
  build-script check) to confirm every extracted component compiles and
  every callback wiring resolves.
- Visual smoke-test: open Discover → Home, switch between Home/Editor's
  Picks/For You/Recommendations tabs, scroll the section list, verify every
  section still renders (new releases, playlists with tag filter, recently
  played albums/tracks with refresh, popular, library albums with sort,
  most played, mixes, release watch, top artists, pinned) and the genre
  filter popup still opens/closes correctly.
