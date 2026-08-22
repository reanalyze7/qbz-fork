# crates/qbz/src/folders.rs (236 lines)

## Summary
Playlist folder organization (library.db-backed, flat/no-nesting): basic
sidebar helpers (`FolderInfo`, folder/position maps) plus a richer
"Playlist Manager" API (`FolderFull`, hidden/favorite flags, stats, custom
sort). All ops are blocking DB calls via `library_db::with_db`.

## Proposed split
The file already banner-separates "sidebar" from "Playlist Manager" —
split exactly along that line:

- `folders/mod.rs` (~10 lines) — `pub use` of `sidebar` and `manager`.
- `folders/sidebar.rs` (~80 lines) — `FolderInfo`, `load_folders`,
  `playlist_folder_map`, `playlist_positions`, `create_folder`,
  `delete_folder`, `move_playlist`, `move_local_playlist` (the lightweight
  id+name API the sidebar controller uses).
- `folders/manager.rs` (~150 lines) — `FolderFull`,
  `PlaylistSettingsLite`, `load_folders_full`, `playlist_settings_map`,
  `playlist_play_counts`, `playlist_local_counts`, `create_folder_full`,
  `update_folder_full`, `set_favorite`, `set_hidden`, `set_folder_hidden`,
  `reorder_playlists`. Still over — split the read-side (`load_folders_full`
  through `playlist_local_counts`, ~65 lines) from the write-side
  (`create_folder_full` through `reorder_playlists`, ~85 lines) into
  `folders/manager/read.rs` and `folders/manager/write.rs` if it needs to
  fit strictly under 130.

## Re-export surface
`folders/mod.rs` stays the `mod folders;` target; `pub use sidebar::*; pub
use manager::*;` keeps every function at `crate::folders::X` — `sidebar.rs`
(the controller) and the Playlist Manager screen both call these paths
unchanged.

## Coupling / watch out
- All functions are thin wrappers around `library_db::with_db(|db|
  db.method(...))` — no logic beyond struct field mapping. This is one of
  the safest/most mechanical splits in the batch; low risk.
- `FolderInfo` (sidebar.rs) and `FolderFull` (manager.rs) are DIFFERENT
  structs (id+name vs full icon/color/hidden record) despite similar
  names — do not merge them; `crate::sidebar.rs` imports `FolderInfo`
  specifically (`use crate::folders::FolderInfo;`).
- `delete_folder` does TWO separate `with_db` calls (one for the Qobuz-side
  FK cascade, one to null local members' `folder_id` because the app keeps
  `foreign_keys` pragma off) — keep both calls, in order, in the same
  function; the comment explaining why is load-bearing documentation.

## Verify after split
- `cargo build -p qbz` (no `#[cfg(test)]` in this file).
- Smoke-test: create/delete a folder, move a Qobuz playlist and a local
  playlist into/out of a folder, and the Playlist Manager's hide/favorite/
  reorder actions.
