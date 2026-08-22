# crates/qbz/src/playlist_manager.rs (1043 lines)

## Summary
The Playlist Manager controller (Tauri `PlaylistManagerView`'s Slint
equivalent): loads Qobuz + local playlists + folders, merges them into
Send row structs, applies toolbar search/filter/sort/view-mode, builds the
grid/list/tree Slint models, and handles optimistic local mutations
(favorite/hidden/folder/reorder toggles) plus artwork job dispatch.

## Proposed split
By domain (data model+load / sort+filter / model builders / rebuild+render /
artwork / mutations / navigation):

- `playlist_manager/mod.rs` (~40 lines) — module doc, imports, `mod` wiring
  (`mod types; mod load; mod sort_filter; mod build; mod rebuild; mod
  artwork; mod mutate; mod navigate;`), re-exports the public functions/types
  so `crate::playlist_manager::{load, apply, rebuild, ...}` call sites are
  unchanged.
- `playlist_manager/types.rs` (~80 lines) — `PmPlaylist`, `PmLocalPlaylist`,
  `PmData`, the `CACHE`/`EXPANDED`/`TREE_INIT` statics, `cover_urls` helper.
- `playlist_manager/load.rs` (~185 lines, still over — split into
  `load.rs` (~110: the `load()` async fn body up to the internal-favorites
  merge) and `load_offline.rs` (~75: the "surface internal favorites" +
  "D11.b offline synthesis" blocks as two helper fns called from `load()`)).
- `playlist_manager/sort_filter.rs` (~120 lines) — `sort_playlists`,
  `PmEntry` enum + impl, `sort_entries`, `local_entries`, `passes`.
- `playlist_manager/build.rs` (~100 lines) — `parse_color`, `format_duration`,
  `folder_item`, `playlist_item`, `local_playlist_item` (pure row builders).
- `playlist_manager/rebuild.rs` (~130 lines) — `set_loading`, `apply`,
  `reset_session`, `rebuild`, `search_menu_folders`, `build_tree`,
  `toggle_tree_folder` (the render/rebuild pipeline; `build_tree` alone is
  ~90 lines and could move to its own `tree.rs` if this file runs long).
- `playlist_manager/artwork.rs` (~90 lines) — `artwork_jobs`,
  `load_folder_custom_images`, `decode_local_image`, `folder_for_edit`,
  `load_editor_custom_image`, `set_folder_image`.
- `playlist_manager/mutate.rs` (~115 lines) — `toggle_favorite_local`,
  `toggle_hidden_local`, `toggle_local_favorite`, `toggle_local_hidden`,
  `move_to_folder_local`, `move_up`, `move_down`, `reorder_step`.
- `playlist_manager/navigate.rs` (~25 lines) — `navigate`.

## Re-export surface
`playlist_manager/mod.rs` — this file becomes
`crates/qbz/src/playlist_manager/mod.rs`. It must `pub use` (or keep as
plain `pub fn` defined via `mod` re-export) every function currently called
as `crate::playlist_manager::X` from elsewhere in the `qbz` crate: `load`,
`set_loading`, `apply`, `reset_session`, `rebuild`, `search_menu_folders`,
`toggle_tree_folder`, `artwork_jobs`, `load_folder_custom_images`,
`folder_for_edit`, `load_editor_custom_image`, `toggle_favorite_local`,
`toggle_hidden_local`, `toggle_local_favorite`, `toggle_local_hidden`,
`move_to_folder_local`, `move_up`, `move_down`, `navigate`, plus the `PmData`
type (used by `apply`'s caller). Search the crate for
`playlist_manager::` call sites before finalizing which need `pub use` vs.
staying module-qualified (`playlist_manager::rebuild::rebuild` would break
callers using `playlist_manager::rebuild` — needs `pub use rebuild::rebuild;`
in mod.rs, or just keep `rebuild` function directly in mod.rs to avoid a
name clash between the module `rebuild` and the function `rebuild`).
- NOTE: because one submodule is named `rebuild` and it contains a function
  ALSO named `rebuild`, prefer renaming the module to `render.rs` to avoid the
  `rebuild::rebuild` stutter/shadow-naming footgun.

## Coupling / watch out
- `CACHE`, `EXPANDED`, `TREE_INIT` (module-level `LazyLock<Mutex<...>>`
  statics in `types.rs`) are read/written from `load`/`rebuild`/`mutate`/
  `artwork` — must be `pub(super)` (or `pub(crate)`) so every submodule can
  reach them; they are today file-private (`static CACHE: ...` with no `pub`)
  so this is a visibility change, not just a move.
- `PmEntry` (in `sort_filter.rs`) borrows `&PmPlaylist`/`&PmLocalPlaylist`
  from `types.rs` and calls `playlist_item`/`local_playlist_item` from
  `build.rs` inside its `item()` method — creates a `sort_filter -> build`
  dependency; keep `build.rs` free of `sort_filter` imports to avoid a cycle.
- `rebuild()` (in the renamed `render.rs`) calls `build_tree`, which itself
  calls `local_entries` + `sort_entries` (from `sort_filter.rs`) AND
  `folder_item`/`playlist_item` (from `build.rs`) — three-way fan-in, keep
  `render.rs` as the integration point, not `sort_filter.rs` or `build.rs`.
- `navigate()` (navigate.rs) calls `load`, `apply`, `artwork_jobs`,
  `load_folder_custom_images`, `reset_session`, `set_loading` — it is the
  only cross-submodule "glue" function; keep it last in the mod graph so it
  can freely depend on everything else.
- `TREE_INIT`/`EXPANDED` session state means `build_tree` is NOT pure — it
  mutates `TREE_INIT` on first call (auto-expand). Don't accidentally
  duplicate that static across files if refactoring further; there must be
  exactly one `TREE_INIT`.
- `qbz_i18n::t_args`/`tf` calls and the `AppWindow`/`PlaylistManagerState`
  Slint globals are used throughout `render.rs`/`build.rs`/`mutate.rs` — no
  special coupling risk, just note these come from the `crate::{AppWindow,
  ...}` glob import at the top of the current file, which each split file
  will need to repeat/import individually.

## Verify after split
- `cargo check -p qbz` (or whichever crate name `crates/qbz` builds as) —
  confirms no broken `crate::playlist_manager::X` call sites.
- `cargo test -p qbz` if any unit tests reference these functions (none seen
  in this file itself, but check callers under `crates/qbz/src/` for
  integration-style tests).
- Smoke-test: open the Playlist Manager view in the running app (grid/list/
  tree modes, search, sort, folder create/toggle, favorite/hide toggle,
  custom-order reorder) — this file has no automated UI tests, so a manual
  pass matters more than usual here.
