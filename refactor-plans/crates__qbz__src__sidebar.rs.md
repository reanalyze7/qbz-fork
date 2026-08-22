# crates/qbz/src/sidebar.rs (749 lines)

## Summary
Sidebar playlists + folders controller: builds the flattened left-nav list
(folder headers + their playlists + root playlists, Qobuz AND local),
handles sort/search/expand/move, offline visibility filtering (D11.b/B7/B8),
and artwork job derivation. Session-cached so expand/move rebuild without
refetching.

## Proposed split
By concern — data model/load vs render/rebuild vs row-building vs mutation
callbacks:

- `sidebar/mod.rs` (~90 lines) — `SidebarPlaylist`, `LocalSidebarPlaylist`,
  `SidebarData` structs, the five `static`s (`EXPANDED`, `CACHE`, `SORT`,
  `SEARCH`, `NAME_DESC`), `pub use` of submodules.
- `sidebar/load.rs` (~135 lines) — `playlist_cover_urls`, `load` (the big
  async fetch: Qobuz playlists + folders + local playlists + offline
  snapshot synthesis). Still over — split the offline-synthesis tail
  (D11.b/B7/B8 block building synthetic entries from `local_counts`/
  `snapshot_available`/`snapshot_names`, ~70 lines) into
  `sidebar/load_offline.rs`, leaving the online fetch + blocking-DB-load
  (~65 lines) in `load.rs`.
- `sidebar/sort_search.rs` (~40 lines) — `set_sort`, `set_search`,
  `sort_playlists`.
- `sidebar/rebuild.rs` (~130 lines) — `rebuild` (the big flattened-list
  builder: folders + their members + root playlists + root locals). At
  budget; if it grows, split the folder-loop body out as a helper fn.
- `sidebar/entry_build.rs` (~65 lines) — `playlist_entry`,
  `local_playlist_entry` (the two `SidebarEntry` row builders).
- `sidebar/offline_filter.rs` (~10 lines) — `offline_visible`.
- `sidebar/folder_popup.rs` (~35 lines) — `load_folder_popup`.
- `sidebar/lookups.rs` (~40 lines) — `local_playlist_meta`,
  `playlist_name_desc`, `playlist_track_count`, `set_active`,
  `search_menu_folders` (small read-only accessors + trivial setters).
- `sidebar/mutate.rs` (~55 lines) — `apply`, `rename_entry`,
  `toggle_folder`, `move_playlist_local`, `move_local_playlist_local` (cache
  mutation + rebuild triggers).
- `sidebar/artwork.rs` (~30 lines) — `artwork_jobs`.

## Re-export surface
`sidebar/mod.rs` stays the `mod sidebar;` target with the three structs +
five statics defined there. Every function used by `main.rs` (`load`,
`apply`, `rebuild`, `set_sort`, `set_search`, `toggle_folder`,
`move_playlist_local`, `move_local_playlist_local`, `rename_entry`,
`set_active`, `search_menu_folders`, `artwork_jobs`, `load_folder_popup`,
`local_playlist_meta`, `playlist_name_desc`, `playlist_track_count`) stays
reachable at `crate::sidebar::X` via `pub use` chains from each submodule.

## Coupling / watch out
- `CACHE`, `EXPANDED`, `SORT`, `SEARCH`, `NAME_DESC` are all shared
  process-global statics touched from MOST of the split files
  (`rebuild.rs`, `sort_search.rs`, `mutate.rs`, `folder_popup.rs`,
  `lookups.rs` all read/write at least one) — they must stay defined in
  `mod.rs` with every submodule doing `use super::{CACHE, EXPANDED, ...};`.
  This is the single biggest cross-file coupling point in this split.
- `rebuild()` and `load_folder_popup()` independently re-implement very
  similar folder-member-filtering logic (sort, folder_map lookup, hidden
  check, `offline_visible`, local-members-by-folder-sorted) — this is
  pre-existing duplication, not something to silently merge during a
  mechanical split; flag it for the real split PR as an extraction
  opportunity (a shared `folder_members(data, folder_id, ...)` helper).
- `offline_visible` (tiny, 5 lines) is called from BOTH `rebuild.rs` and
  `folder_popup.rs` — keep it as a free `pub(super)` fn in its own tiny
  file (or in `mod.rs`) so both can `use super::offline_visible;`.
- The D11.b/B7/B8 offline-playlist-synthesis block in `load()` is dense,
  spec-numbered logic (comments reference "D11.b", "B7", "B8" throughout) —
  preserve every comment verbatim when moving to `load_offline.rs`, they
  are the only documentation of WHY this synthesis exists.
- `NAME_DESC` is both a load-time cache (populated in `load()`) AND
  optimistically patched by `rename_entry` — the two writers must agree on
  format; no behavior change intended, just correct file placement.

## Verify after split
- `cargo build -p qbz` (no `#[cfg(test)]` in this file — flag as a gap;
  this is complex enough that a real split PR should add unit tests for
  `sort_playlists`, `offline_visible`, and the folder-filtering predicates
  before touching the file).
- Smoke-test: sidebar load online AND offline (verify D11.b/B7/B8 synthetic
  playlist rows appear correctly), folder expand/collapse, drag playlist
  into/out of folder, sort options, search filter, and the collapsed-
  sidebar folder popup flyout.
