# crates/qbz/src/local_playlist.rs (1829 lines)

## Summary
Slint-side controller for LOCAL playlists (`local:<uuid>` ids in the shared
`library.db`): blocking repo CRUD wrappers, the detail-view load/resolve/apply
pipeline shared with the offline-rendered MIXED Qobuz playlist detail,
playback (play-all/play-from-row/enqueue), drag-reorder, multi-select
removal, and "Upload to Qobuz" migration.

## Proposed split
Turn into a `local_playlist/` directory, one module per concern:

- `local_playlist/mod.rs` (~70 lines) — module doc, `PlaylistRef` enum +
  `parse`, `is_local_id`, `Runtime` type alias, `pub use` re-exports of every
  `pub fn`/`pub` item from the submodules below so `crate::local_playlist::X`
  keeps working unchanged for every caller.
- `local_playlist/repo.rs` (~230 lines) — lines 52-340: all the "blocking repo
  wrappers" (`list_blocking`, `get_blocking`, `get_tracks_blocking`,
  `resolve_cover_urls`, `create_blocking`, `update_blocking`,
  `delete_blocking`, `set_favorite_blocking`, `set_hidden_blocking`,
  `add_qobuz_tracks_blocking`, `local_row_input`, `add_inputs_blocking`,
  `add_local_refs_blocking`, `add_drag_tracks_blocking`,
  `set_custom_artwork_blocking`, `clear_custom_artwork_blocking`).
- `local_playlist/state.rs` (~40 lines) — the three open-detail statics
  (`CURRENT_QUEUE`, `CURRENT_META`, `ROW_POSITIONS`) plus their small
  accessors that don't belong to a specific detail flow
  (`queue_track_for_row`, `local_picker_ref_for_row`,
  `set_open_mixed_snapshot`, `clear_open_snapshot`) — kept together because
  every other module reads/writes these three statics.
- `local_playlist/row.rs` (~260 lines) — lines 376-451 + 716-976: `RowItem`
  enum, `LoadedRow`, `LocalPlaylistData`, `mmss`, `total_duration_label`,
  `row_queue_track`, `row_item`, `build_row_models` — the shared row-identity
  contract (E11) used by both the LOCAL detail and the offline MIXED detail.
- `local_playlist/detail_local.rs` (~230 lines) — lines 453-1106 minus the
  row/state pieces already moved: `read_sidecar_rows_blocking`, `load`,
  `artwork_jobs`, `apply`, `navigate` (the LOCAL playlist detail open/apply
  flow).
- `local_playlist/detail_offline_mixed.rs` (~200 lines) — lines 1108-1288:
  `navigate_qobuz_offline`, `apply_qobuz_offline` (the offline-rendered MIXED
  Qobuz playlist detail — D11.a).
- `local_playlist/playback.rs` (~150 lines) — lines 1290-1432: `play_stamped`,
  `visible_ordered_queue`, `play_all`, `play_from_visible`, `enqueue_by_id`.
- `local_playlist/reorder.rs` (~200 lines) — lines 1434-1637: `move_row`,
  `reorder_row` (B2 drag-reorder, both share the position-map transform
  pattern).
- `local_playlist/remove.rs` (~70 lines) — lines 1638-1706: `remove_selected`,
  `remove_rows_by_ids`.
- `local_playlist/upload.rs` (~120 lines) — lines 1707-1829:
  `upload_to_qobuz` (local → real Qobuz playlist migration, D8-aware).

## Re-export surface
`local_playlist/mod.rs` becomes the `mod local_playlist;` target. Every
`pub fn` currently reachable as `crate::local_playlist::foo` (used from
`main.rs`, `playlist.rs`, sidebar/drag code, etc.) must stay reachable at that
exact path via `pub use repo::*; pub use row::*; pub use playback::*;` etc.
Note `pub(crate) fn local_row_input`, `pub(crate) fn total_duration_label`,
and `pub(crate) fn row_queue_track`/`build_row_models` are `pub(crate)`, not
`pub` — re-export with matching visibility.

## Coupling / watch out
- `CURRENT_QUEUE` / `CURRENT_META` / `ROW_POSITIONS` (in `state.rs`) are read
  and written from `detail_local.rs`, `detail_offline_mixed.rs`,
  `playback.rs`, `reorder.rs`, and `remove.rs` — do NOT duplicate the
  `static` declarations; every other module must `use super::state::*`.
- `RowItem`/`LoadedRow`/`build_row_models` (in `row.rs`) is the E11 shared
  row-identity contract between the LOCAL detail and the offline MIXED
  detail — keep it one module both detail files import, don't fork it.
- `repo::LOCAL_PLAYLIST_PREFIX`, `repo::is_local_playlist_id`, and
  `crate::local_library::LEGACY_SYNTHETIC_ID_FLOOR` are read from multiple
  new submodules (repo.rs, detail_local.rs, detail_offline_mixed.rs) —
  re-import in each rather than funneling through mod.rs.
- The D8 offline-only queue stamp (`set_queue_offline_only`) is read in
  `playback.rs::play_stamped` from `CURRENT_META` and written in
  `enqueue_by_id`/`detail_local.rs::navigate` — keep the invariant "stamp
  read AFTER `set_queue`, cleared on every replacement" intact across the
  split.
- Cross-file position-map transform logic (`move_row`/`reorder_row` in
  `reorder.rs`) duplicates the same shift arithmetic in both functions —
  tempting to extract a shared helper during the split, but that's a
  behavior-risk refactor beyond scope; leave as-is unless explicitly asked.

## Verify after split
- `cargo check -p qbz` and `cargo build -p qbz` (no dedicated unit tests in
  this file; check for `#[cfg(test)]` — none found here).
- Manually smoke-test: open a local playlist, open an offline-rendered mixed
  Qobuz playlist while offline, play-all/shuffle/play-from-row, drag-reorder
  a row, multi-select remove, and "Upload to Qobuz" end-to-end (verify the
  local entity is deleted only on full success, kept on partial failure).
