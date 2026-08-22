# crates/qbz/src/favorites.rs (1674 lines)

## Summary
Library > Favorites controller: fetches Tracks/Albums/Artists/Playlists/Labels via
`QbzCore::get_favorites`, maps JSON into typed Slint row/card structs, applies them
to `FavoritesState`, and owns per-tab derive (search/sort/group), un-favorite fade,
artwork propagation, multi-select, and a windowed artwork pipeline for the Albums tab.

## Proposed split
By responsibility, following the file's own section comments (`---- ... ----`).

- `favorites/mod.rs` (~60 lines) — module doc, imports, `pub const PAGE_SIZE`,
  `const MAX_ITEMS`, `FavTab` enum + its `from_route`/`from_tab_id`/`key`, and
  `pub use` re-exports of every public fn/type from the split files below, so
  `crate::favorites::apply_favorites` etc. keep working unchanged.
- `favorites/fetch.rs` (~230 lines) — `favorite_album_ids`, `load_favorites` (the
  playlists sub-tab branch + the generic paginated fetch), `FavData` enum,
  `FavLabel` struct. The network/parsing "IO" layer.
- `favorites/counts.rs` (~60 lines) — `FavCounts`, `total_for`, `load_counts`,
  `apply_counts`. Small, self-contained.
- `favorites/mapping.rs` (~110 lines) — `TrackCard`/`ArtistCard`/`LabelCard` structs,
  `mmss`, `map_track`, `map_artist`, `map_label`. Pure data-shaping, no Slint models.
- `favorites/apply.rs` (~140 lines) — `apply_favorites` (the big match over
  `FavData` that pushes into `FavoritesState`). Likely still needs a further split
  by tab arm (tracks/albums/artists/playlists/labels) if it stays over 130 once
  isolated — consider `apply_tracks`/`apply_albums`/etc. private helper fns in the
  same file to keep it readable without another file.
- `favorites/derive.rs` (~330 lines) — `derive_tracks`, `derive_labels`,
  `derive_playlists`, `derive_artists`, `derive_albums`, `album_alpha_key`,
  `album_genre_matches`, `track_genre_matches`. This is the single biggest chunk;
  split further into `derive/tracks.rs`, `derive/artists.rs`, `derive/albums.rs`
  (albums alone is ~110 lines) sharing the alpha-key/genre-match helpers via a
  `derive/mod.rs` or a `favorites/filter_helpers.rs`.
- `favorites/albums_artwork.rs` (~230 lines) — the whole "Windowed albums artwork"
  section: `FAV_ALBUMS_GEN`, `albums_gen_current`, `FAV_ALBUMS_WINDOW`,
  `fav_albums_inflight`, `fav_albums_dispatch_ctx`, `album_artwork_job_done`,
  `begin_albums_artwork`, `FAV_ALBUMS_DISPATCH_THROTTLE_MS`, `FAV_ALBUMS_BAND`,
  `albums_window_changed`, `dispatch_fav_albums_window`,
  `dispatch_fav_albums_all_visible`, `albums_view_mode_changed`,
  `dispatch_fav_albums_all_grouped`. Self-contained subsystem, all its statics are
  file-local — good split boundary.
- `favorites/random.rs` (~70 lines) — `random_visible_album`, `random_visible_artist`,
  `random_visible_playlist`, `random_visible_label`, plus `play_tracks`/
  `shuffled_tracks` (the `FAV_CURRENT` static and its accessors) since they share
  the "pick from currently visible/loaded set" theme.
- `favorites/mutate_rows.rs` (~130 lines) — the "Un-favorite in place" section:
  `mark_track_removing`, `remove_track_row`, `mark_album_removing`,
  `remove_album_row`, `remove_playlist_row`.
- `favorites/artwork_apply.rs` (~90 lines) — `set_artwork_in_albums`,
  `set_album_artwork`, `set_artist_image`, `set_playlist_cover`, `set_track_artwork`.
- `favorites/selection.rs` (~90 lines) — the "Tracks multi-select" section:
  `set_multi_select`, `recount_selected`, `select_all`, `clear_selection`,
  `selected_ids`, `selected_tracks`.
- `favorites/selected_artist.rs` (~50 lines) — `apply_selected_artist`,
  `selected_artist_artwork_jobs` (sidepanel artist-albums helpers).
- `favorites/loading.rs` (~10 lines) — `reset_loading`. Could fold into `mod.rs`
  instead of its own file if that keeps `mod.rs` under budget.
- `favorites/artwork_jobs.rs` (~55 lines) — the top-of-load `artwork_jobs` fn
  (builds `ArtworkJob`s per tab for the initial dispatch, distinct from the
  windowed-albums pipeline).

## Re-export surface
`favorites/mod.rs` is the public API surface: `pub use` every item currently
`pub fn`/`pub struct`/`pub enum` at the top level so external callers
(`crate::favorites::load_favorites`, `crate::favorites::apply_favorites`,
`crate::favorites::FavTab`, etc. — called from main.rs / view dispatch code)
need zero changes.

## Coupling / watch out
- `FAV_CURRENT` (Mutex<Vec<Track>>) is written in `apply.rs` (Tracks arm) and read
  by `play_tracks`/`shuffled_tracks` (random.rs) and `selected_tracks` (selection.rs)
  — three different files touching one static; keep it `pub(crate)` in whichever
  file becomes canonical (suggest `random.rs`) and have others `use` it, or hoist to
  `mod.rs` as a shared `pub(super)` static.
- `FAV_TRACK_GENRE` thread_local is written in `apply.rs` and read in
  `derive.rs::track_genre_matches` — same cross-file coupling; keep both aware of
  each other via a shared location (e.g. define in `mod.rs`, `pub(super)`).
- The Albums-tab windowed artwork pipeline (`albums_artwork.rs`) is invoked FROM
  `derive.rs::derive_albums` (calls `dispatch_fav_albums_window`/
  `dispatch_fav_albums_all_visible`/`dispatch_fav_albums_all_grouped`) — these two
  files are tightly coupled; document the call direction so a future refactor
  doesn't try to merge them awkwardly or break the generation-guard invariants.
- `apply_favorites`'s Albums arm interacts with `begin_albums_artwork` (called by
  the CALLER before `apply_favorites`, per its doc comment) — this ordering
  contract must survive the split (document in `apply.rs` doc comment).
- Multiple `derive_*` functions each independently duplicate the alpha-key /
  genre-match helper pattern; keep `album_alpha_key` and the two `*_genre_matches`
  fns as `pub(super)` in one shared spot (e.g. `favorites/filter_helpers.rs`) so
  `derive.rs`'s sub-files don't each reimplement them.

## Verify after split
- `cargo build -p qbz` (or workspace) — favorites.rs is called from main.rs /
  view-dispatch, so a broken re-export breaks the app binary.
- `cargo test -p qbz` if any unit tests exist for favorites helpers (none seen in
  this file, but check dependents).
- Smoke-test: `grep -rn "favorites::" crates/qbz/src` still resolves; specifically
  `favorites::load_favorites`, `favorites::apply_favorites`, `favorites::FavTab`,
  `favorites::begin_albums_artwork`, `favorites::albums_window_changed`.
- Manual/UI smoke-test (or slint-viewer if applicable): open Favorites, switch all
  five tabs, search/sort/group each, scroll the Albums grid (windowed artwork),
  un-favorite a track/album, multi-select tracks, shuffle/play-all.
