# crates/qbz/src/artist.rs (1927 lines)

## Summary
Artist detail controller: fetches an artist page (+ releases pagination, Magazine
stories, MusicBrainz origin/relationships/discovery) through `QbzCore`, maps raw API
responses to plain `Send` data on a worker thread, and applies/mutates the
`ArtistState` / `NetworkSidebarState` Slint globals on the UI event loop. Also owns
multi-select and in-page search for the Popular Tracks list.

## Proposed split
By domain — this file already reads as five loosely-coupled sub-features glued only
by `ArtistState`/`AppWindow`. Split into a `artist/` directory, one `mod.rs` re-export.

- `artist/mod.rs` (~20 lines) — module doc + `pub use` of every public item from the
  submodules below, so `crate::artist::X` paths are unchanged for callers (main.rs,
  album.rs, etc.).
- `artist/data.rs` (~110 lines) — the plain data structs: `ArtistData`, `PlaylistSlim`,
  `LabelData`, `SimilarArtistData`, `ReleaseSection`, `StoryData`, `RELEASE_SECTION_ORDER`,
  `release_type_title`, `title_case`.
- `artist/load.rs` (~130 lines) — the async fetch/map functions that build `ArtistData`:
  `load_artist`, `load_release_page`, `map_artist`, `truncate_words`, `RELEASE_PAGE_SIZE`.
  (`map_artist` alone is ~220 lines currently — see note below, may need its own file
  `artist/map_artist.rs` ~230 lines if kept whole, or split further into
  `map_releases_bucket` / `map_playlists` helpers to fit under 130.)
- `artist/stories.rs` (~60 lines) — `StoryData` load/apply: `map_story`, `load_stories`,
  `apply_stories`.
- `artist/track_map.rs` (~110 lines) — `map_track`, `map_release`, `tier`, `mmss`,
  `card_to_item`, `track_data_to_item`, `playlist_to_item` (the Slint-item mapping
  helpers shared by several apply functions).
- `artist/artwork_jobs.rs` (~60 lines) — `artwork_jobs` (cover-job collection over
  `ArtistData`).
- `artist/apply.rs` (~130 lines) — `apply_artist` (the big state-application function;
  currently ~125 lines alone) plus the `FULL_TOP_TRACKS`/`FULL_APPEARS_ON`/
  `FULL_RELEASE_SECTIONS`/`LOADED_PAGES` thread_locals and `MAX_INDEX_PAGES` constant
  (these are read/written from `apply.rs`, `search.rs`, `sort.rs`, `paging.rs` — see
  coupling note; consider moving thread_locals into their own tiny `artist/cache.rs`
  ~30 lines to avoid every consumer file re-declaring them).
- `artist/search.rs` (~60 lines) — `filter_artist` (in-page search) + `build_jump_tabs`
  (jump-tab construction with its layout constants) — or split `build_jump_tabs` into
  its own `artist/jump_tabs.rs` (~90 lines) since it's a distinct pure-computation
  concern from search filtering.
- `artist/favorites.rs` (~80 lines) — `set_release_card_favorite`, `set_release_card_pinned`,
  `reset_artist`.
- `artist/sort_page.rs` (~120 lines) — `resort_section`, `section_loaded_count`,
  `section_can_load_more`, `append_release_page` (the load-more/paging + sort logic).
- `artist/multi_select.rs` (~80 lines) — `set_multi_select`, `recount_selected`,
  `select_all`, `clear_selection`, `selected_ids`, `all_top_track_ids`.
- `artist/artwork_apply.rs` (~20 lines) — `apply_artwork`.
- `artist/musicbrainz.rs` (~130 lines) — `MbMetadata`, `MbOrigin`, `reset_network_sidebar`,
  `load_mb_metadata`, `map_origin`, `LocationParams`, `LOCATION_PARAMS` static,
  `store_location_params`, `location_params`, `apply_mb_metadata`, `apply_mb_unavailable`,
  `format_mb_date_short`. This is itself ~350 lines in the original file — split further:
  - `artist/mb/origin.rs` (~130 lines) — `MbMetadata`, `MbOrigin`, `load_mb_metadata`,
    `map_origin`, `apply_mb_metadata`, `apply_mb_unavailable`, `format_mb_date_short`.
  - `artist/mb/location.rs` (~60 lines) — `LocationParams`, `LOCATION_PARAMS`,
    `store_location_params`, `location_params`, `reset_network_sidebar`.
  - `artist/mb/relationships.rs` (~130 lines) — `MbRelationshipsRowData`,
    `MbRelationshipRow`, `load_mb_relationships`, `map_relationships`, `group_relations`,
    `format_period`, `apply_mb_relationships`.
  - `artist/mb/discovery.rs` (~100 lines) — `MbDiscoveryData`, `MbDiscoveryRow`,
    `load_mb_discovery`, `apply_mb_discovery`, `remove_discovery_artist`.

## Re-export surface
`artist/mod.rs` re-exports every public struct/fn so all existing `crate::artist::*`
call sites (in `main.rs` and elsewhere in `qbz`) compile unchanged. No renaming of any
public item.

## Coupling / watch out
- `map_release` is `pub(crate)` and used both by `map_artist` (in load.rs) and directly
  by other controllers (e.g. label.rs likely calls `artist::map_release`) — keep it
  re-exported at crate visibility from `artist/mod.rs`.
- `card_to_item` is also `pub(crate)` — check other controllers for direct references
  before moving.
- The four `thread_local!` caches (`FULL_TOP_TRACKS`, `FULL_APPEARS_ON`,
  `FULL_RELEASE_SECTIONS`, `LOADED_PAGES`) are written in `apply_artist` (apply.rs),
  read/written in `filter_artist` (search.rs), `resort_section`/`append_release_page`
  (sort_page.rs), and cleared in `reset_artist` (favorites.rs) — this is the single
  biggest cross-file coupling risk. Putting them in a dedicated `artist/cache.rs`
  module with thin accessor functions (rather than raw `thread_local!` in each file)
  avoids duplicated `.with(...)` boilerplate and keeps the mutable state in one place.
- `LOCATION_PARAMS` static Mutex is written by `store_location_params` and read by
  `location_params()`, called from a click handler in `main.rs` — keep both functions
  in the same file (`mb/location.rs`).
- `set_release_card_favorite`/`set_release_card_pinned` both touch the FULL cache too —
  keep them able to import from `artist::cache` (or wherever the thread_locals land).
- Heavy reliance on sibling modules (`crate::album::TrackData`, `crate::artwork`,
  `crate::home::CardData`, `crate::fav_cache`, `crate::pinned`, `crate::selection`,
  `crate::artist_blacklist`, `crate::artist_prefs`, `crate::album_map`,
  `crate::custom_artwork`, `crate::immersive`, `crate::discovery_dismiss`,
  `crate::play_history`, `crate::reco`, `crate::strip_html`) — these are all outside
  this file's scope but every submodule created must re-import what it needs; no new
  crate-level imports are introduced, just re-distributed `use` statements.

## Verify after split
- `cargo check -p qbz` and `cargo build -p qbz` (this crate has no visible unit tests
  in this file, but downstream artist-page flows are integration-tested elsewhere).
- Manual/smoke test: open an artist page, verify Popular Tracks, release sections
  (sort + load-more), Magazine stories, and the Network sidebar (Origin/Relationships/
  Discovery) all still populate and update correctly.
- Grep the whole `qbz` crate (and `qbzd` if it links this crate) for
  `artist::map_release`, `artist::card_to_item`, `artist::MAX_INDEX_PAGES`,
  `artist::RELEASE_PAGE_SIZE` and any other cross-file references to confirm nothing
  broke after the split.
