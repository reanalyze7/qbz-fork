# crates/qbz/src/local_library.rs (3547 lines)

## Summary
Slint-side controller for the Local Library view: per-tab (Albums/Folders/
Tracks/Artists) data loading against `qbz-library`'s `library.db`, client-side
derive/filter/sort/group, windowed artwork dispatch, multi-select, album
detail, folder tree, artist merge, and an "ephemeral folder" (outside-library
browse) mode. By far the largest file in this pass — needs a by-domain split,
one file per tab/concern, not a mechanical chunking.

## Proposed split
Turn into a `local_library/` directory, one module per section (sections
are already marked with `// ==== ... ====` banner comments in the source,
which makes the split boundaries unambiguous):

- `local_library/mod.rs` (~60 lines) — module doc, shared imports, `LibTab`
  enum + its `from_route`/`from_tab_id`/`tab_id` methods (used by every tab),
  `pub use` re-exports of every `pub fn` from the submodules below, plus the
  couple of cross-tab helpers that don't belong to one tab
  (`exclude_network_folders_now`, `reset_browse_models`,
  `LEGACY_SYNTHETIC_ID_FLOOR`, `fetch_album_tracks_blocking`).
- `local_library/albums.rs` (~500 lines, still over budget — see note below)
  — lines 72-884: `map_local_album`, `local_quality`, `LOCAL_ALBUMS` +
  `local_albums()`, `AlbumFilter`/`read_album_filter`/`album_filter_count`/
  `album_matches_filters`, the windowed-artwork dispatch machinery
  (`ALBUMS_WINDOW`, `albums_inflight`, `albums_dispatch_ctx`,
  `album_artwork_job_done`, `albums_window_changed`, `dispatch_albums_window`,
  `set_local_album_artwork`, `set_local_folder_artwork`), `derive_albums`,
  `dispatch_albums_all_visible`, `albums_view_mode_changed`,
  `dispatch_albums_all_grouped`, `clear_album_filter`, `current_group_mode`,
  `spawn_albums_load`, `reload_albums`, `seed_counts`, `ensure_albums_loaded`.
  This one section alone is ~800 lines — split it FURTHER into:
  - `albums/load.rs` (~260 lines) — map/filter/LOCAL_ALBUMS/derive/spawn_load/
    reload/ensure_loaded/seed_counts (the "get data in" half).
  - `albums/artwork.rs` (~230 lines) — the windowed-dispatch machinery +
    set_local_album_artwork/set_local_folder_artwork (the "paint covers" half).
  - `albums/select.rs` (~180 lines) — lines 1215-1550: the Albums
    multi-select block (`set_albums_multi_select`, `rendered_album_ids`,
    `for_each_album_model`, `set_albums_selected`, `album_is_selected`,
    `toggle_album_favorite`, `recount_albums_selected`, `toggle_album_select`,
    `select_all_albums*`, `clear_albums_selection`, `selected_album_ids`,
    `selected_albums_tracks_blocking`).
- `local_library/tracks.rs` (~330 lines) — lines 885-1214 + 1467-1550: the
  Tracks tab (paging, `map_local_track`, `fetch_tracks_page`, apply/append,
  derive_tracks, group/sort setters, multi-select toggle/select-all/clear,
  `selected_local_tracks`, `local_track_by_id`, `spawn_tracks_page_load`,
  `reload_tracks`, `ensure_tracks_loaded`, `load_more_tracks`).
- `local_library/album_detail.rs` (~300 lines) — lines 1551-1864: Album
  detail (`fmt_album_duration`, `ALBUM_VERSIONS`, `ALBUM_QUERY`,
  `open_local_album`, `current_album_disc_tracks`, version picker).
- `local_library/folders_flat.rs` (~130 lines) — lines 1865-2021: flat
  Folders tab load/derive (`ensure_folders_loaded`, folder mapping).
- `local_library/folders_tree.rs` (~350 lines) — lines 2022-2573: folder
  tree (`FolderNode` mapping, `toggle_folder_node`, tree multi-select
  `toggle_tree_folder_select`/`toggle_tree_track_select`, `select_folder`).
- `local_library/artists.rs` (~350 lines) — lines 2574-3286: Artists tab
  (`take_pending_artist`, `build_artist_album_ids`, `merge_artists`,
  `artists_img_gen_current`, `ensure_artists_loaded`, artwork dispatch for
  artist rows).
- `local_library/ephemeral.rs` (~260 lines) — lines 3288-end: the "Open
  Folder" outside-library browse mode (folder_display_name, ephemeral scan,
  album grouping, playback list building).

## Re-export surface
`local_library/mod.rs` becomes the `mod local_library;` target already
referenced from `crates/qbz/src/main.rs` (or wherever it's declared) — every
`pub fn` currently callable as `crate::local_library::foo` must stay callable
at that exact path via `pub use albums::*; pub use tracks::*;` etc. in
`mod.rs`. No caller outside this file should need to change an import.

## Coupling / watch out
- Heavy use of process-global `static`/`LazyLock<Mutex<...>>` caches
  (`LOCAL_ALBUMS`, `TRACKS_CURRENT`, `ALBUM_VERSIONS`, `ALBUM_QUERY`,
  `ALBUMS_GEN`/`TRACKS_GEN` atomics, `albums_inflight()`,
  `albums_dispatch_ctx()`) — these must stay in the module that owns the
  tab they back (do NOT accidentally duplicate them by splitting a `static`
  declaration away from its accessor fn).
- `crate::album_map`, `crate::artwork`, `crate::selection`, `crate::quality`,
  `crate::library_db`, `crate::locallibrary_prefs`, `crate::local_favorites`,
  `crate::keybindings`, `crate::offline_mode` are all cross-cutting deps used
  from multiple new submodules — re-import in each, don't try to funnel
  through mod.rs.
- `LocalLibraryState`/`AppWindow` (Slint generated globals) are used
  everywhere; no special handling needed beyond normal `use` in each file.
- The generation-guard pattern (`ALBUMS_GEN`/`TRACKS_GEN` bumped on reload,
  checked before applying async results) spans load + artwork files within
  the same tab — keep the atomic and every function that reads/writes it in
  the same module (albums: `load.rs`; tracks: `tracks.rs`).
- `exclude_network_folders_now` is read by albums, tracks, folders_flat,
  folders_tree, and artists — keep it in `mod.rs`, not tab-local.

## Verify after split
- `cargo check -p qbz` and `cargo build -p qbz` (this crate has no separate
  test binary listed here, but check for `#[cfg(test)]` blocks that may have
  been missed in this read — grep the file before splitting).
- Manually smoke-test each of the four browse tabs + the ephemeral "Open
  Folder" flow in the running app (windowed artwork scrolling, multi-select,
  album detail version switch, artist merge) since this file has essentially
  no automated test coverage of its own — the split is high risk for typos
  in the generation-guard/static-cache wiring.
