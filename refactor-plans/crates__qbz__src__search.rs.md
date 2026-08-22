# crates/qbz/src/search.rs (1935 lines)

## Summary
Search-results controller: `load_search`/`load_cortinilla`/`load_immersive_search`
run combined Qobuz search (+ local-library search + blacklist filtering) on a
worker thread, `map_*` pure functions turn domain types into plain `Send` row
structs (unit-tested layer), and `apply_*` functions write the `SearchState` /
`ImmersiveState` Slint globals on the event loop. Also owns the "cortinilla"
(live search dropdown) row/section shaping, local-library grouping, load-more
pagination, and artwork-job derivation.

## Proposed split
This is the largest file in the batch (1935 lines) with clear sequential
sections already marked by `// ==== ==== ====` comments in the source — split
along those boundaries into a `search/` directory.

- `search/mod.rs` (~40 lines) — module doc, `pub mod` declarations, `pub use`
  re-exports of every public item so `crate::search::X` paths are unchanged.
- `search/version.rs` (~65 lines) — the three `thread_local!` monotonic
  version counters (`SEARCH_VERSION`, `CORTINILLA_VERSION`,
  `IMMERSIVE_SEARCH_VERSION`) + their `next_*`/`is_current_*` fns.
- `search/rows.rs` (~135 lines) — plain row types: `AlbumRow`, `TrackRowData`,
  `ArtistRow`, `PlaylistRow`, `MostPopularRow`, `SearchData`, `CortRow`,
  `CortSection`, `CortinillaData`, the `CORTINILLA_CAP_*` consts.
- `search/pure.rs` (~100 lines) — pure helpers: `tier`, `quality_label`,
  `mmss`, `year_of`, `playlist_cover_urls`.
- `search/mappers.rs` (~110 lines) — `map_album`, `map_track`, `map_artist`,
  `map_playlist`, `map_most_popular`, `map_search_all` (the unit-tested
  Qobuz-domain -> row mapping layer).
- `search/cortinilla_map.rs` (~300 lines) — `map_search_all_to_cortinilla`,
  `map_search_all_to_immersive`, `LocalCaps` + its impl, the
  IMMERSIVE_CAP_* consts, `assign_flat_indices` (the largest chunk; split
  further into `cortinilla_map/main.rs` + `cortinilla_map/immersive.rs` if it
  still exceeds 130 after extraction — likely needed given ~300 raw lines).
- `search/local_rows.rs` (~180 lines) — local-library row derivation:
  `local_artwork_url`, `local_album_artist`, `derive_local_album_rows`,
  `derive_local_artist_rows`, `map_local_track_to_cort_row`,
  `append_local_sections`, `append_immersive_local_albums`,
  `load_cortinilla_local` (async DB fetch).
- `search/load.rs` (~170 lines) — the async worker-thread loaders:
  `load_search`, `load_cortinilla`, `load_immersive_search`.
- `search/apply.rs` (~230 lines) — Slint event-loop writers: `album_item`,
  `track_item`, `artist_item`, `playlist_item`, `apply_search`,
  `recompute_hi_res_filtered`, `cortinilla_row_item`, `apply_cortinilla`,
  `cortinilla_artwork_jobs`, `apply_immersive_search`,
  `immersive_cortinilla_artwork_jobs`, `reset_search`, `mark_artist_followed`,
  `set_slim_following` — likely still needs a further split (apply.rs vs
  apply_cortinilla.rs) to stay under 130; consider `search/apply/search.rs` +
  `search/apply/cortinilla.rs`.
- `search/pagination.rs` (~200 lines) — `SearchCategory`, `category_for_tab`,
  `search_type_for_filter`, `MoreRows`, `PAGE_SIZE`, `load_more`,
  `append_results`, `replace_category`.
- `search/artwork.rs` (~90 lines) — `simple_job`, `playlist_jobs`,
  `artwork_jobs`, `artwork_jobs_for_more`.
- `search/tests.rs` (~65 lines) — the `#[cfg(test)] mod tests` block
  (`category_for_tab_maps_per_type_tabs`, `search_type_for_filter_...`,
  `mmss_pads_seconds`, `tier_classifies_bit_depth`,
  `quality_label_formats_known_quality`, `map_artist_builds_album_count_subtitle`).

## Re-export surface
`search/mod.rs` re-exports every currently-public item (`AlbumRow`,
`TrackRowData`, `ArtistRow`, `PlaylistRow`, `MostPopularRow`, `SearchData`,
`CortRow`, `CortSection`, `CortinillaData`, `LocalCaps`, `SearchCategory`,
`MoreRows`, all the `map_*`/`load_*`/`apply_*`/`next_*`/`is_current_*`/
`category_for_tab`/`search_type_for_filter` fns) at `crate::search::*` so every
caller across the `qbz` crate (referenced via `crate::search::foo`, e.g. in
`main.rs` click handlers) keeps working unchanged.

## Coupling / watch out
- `playlist_item` is `pub(crate)` (not `pub`) — used from outside this module
  (grep for `search::playlist_item` before finalizing visibility in the split).
- Many functions cross-reference sibling `crate::` modules directly
  (`crate::artist_blacklist`, `crate::fav_cache`, `crate::pinned`,
  `crate::offline_cache`, `crate::library_db`, `crate::search_service`,
  `crate::album_map`, `crate::quality`, `crate::local_library`) — none of
  these need to move, just keep the `use crate::...` paths correct per new
  submodule.
- `to_artist_row`/`to_album_row`/`to_track_row`/`to_playlist_row` closures are
  DUPLICATED almost verbatim between `map_search_all_to_cortinilla` and
  `map_search_all_to_immersive` — if these end up in separate files, resist
  the urge to dedupe during a pure "split" pass (would be a behavior-neutral
  refactor, but plan/execute separately to keep the split mechanical/reviewable).
- `assign_flat_indices` is called from three places (main cortinilla, immersive,
  after local-section append) — keep it a single shared function, not
  duplicated per file.
- The three `thread_local!` version counters are intentionally SEPARATE per the
  module doc (main results page vs main cortinilla vs immersive) — do not
  merge them into one counter during the split.
- `SearchData`/`CortinillaData` field visibility is `pub` — other modules
  likely construct/read these directly (e.g. tests, artwork job builders in
  the same file); keep all fields public after the split.

## Verify after split
- `cargo test -p qbz search::` — all 6 existing unit tests green.
- `cargo check -p qbz` (or full workspace check) — confirms no broken `crate::search::` import paths across `main.rs` and other UI wiring.
- Manual/smoke: run the app, exercise the search results page, the live
  cortinilla dropdown, and the immersive-view search overlay (all three
  consume this file's outputs) to confirm artwork jobs and load-more still
  work after the split.
