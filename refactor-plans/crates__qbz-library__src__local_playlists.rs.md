# crates/qbz-library/src/local_playlists.rs (801 lines)

## Summary
First-class LOCAL (offline, `local:<uuid>`-keyed) playlist CRUD over
SQLite via plain `&Connection` functions — schema/migrations, playlist
header CRUD, track membership CRUD (add/reorder/remove) — plus an
extensive `#[cfg(test)]` block.

## Proposed split
By responsibility: types, schema, playlist CRUD, track CRUD, tests.

- `local_playlists/mod.rs` (~40 lines) — module doc, `pub mod`
  declarations, `pub use` re-exports of every public item so
  `qbz_library::local_playlists::X` paths are unchanged.
- `local_playlists/model.rs` (~90 lines) — `LOCAL_PLAYLIST_PREFIX`,
  `is_local_playlist_id`, `LocalPlaylistTrackSource` (+ `as_str`/`parse`),
  `LocalPlaylist`, `LocalPlaylistTrack`, `LocalPlaylistTrackInput`,
  `now_ms`.
- `local_playlists/schema.rs` (~95 lines) — `init_schema` (the CREATE
  TABLE batch + the two additive-migration guards for `favorite`/`hidden`
  and `folder_id`).
- `local_playlists/playlist_ops.rs` (~125 lines) — the playlist-header CRUD:
  `create`, `rename`, `set_description`, `set_offline_only`, `set_favorite`,
  `set_hidden`, `move_to_folder`, `clear_folder`, `set_custom_artwork`,
  `delete`, `row_to_playlist`, `hydrate_counts`, `list`, `get`.
- `local_playlists/track_ops.rs` (~130 lines) — the membership CRUD:
  `get_tracks`, `add_tracks`, `reorder`, `remove_track`.
- `local_playlists/tests.rs` (~290 lines) — the entire `#[cfg(test)] mod
  tests` block; large enough it may want a further split into
  `tests/playlist_tests.rs` (header CRUD tests) and
  `tests/track_tests.rs` (add/reorder/remove tests + the `qobuz_order`/
  `seeded_playlist` helpers) if 290 lines proves awkward, but a single
  `tests.rs` under ~290 lines is acceptable if the rule is read as
  "non-test code" — check the README's stance on test-file line limits
  before over-splitting.

## Re-export surface
`local_playlists/mod.rs` re-exports `LOCAL_PLAYLIST_PREFIX`,
`is_local_playlist_id`, `LocalPlaylistTrackSource`, `LocalPlaylist`,
`LocalPlaylistTrack`, `LocalPlaylistTrackInput`, `init_schema`, `create`,
`rename`, `set_description`, `set_offline_only`, `set_favorite`,
`set_hidden`, `move_to_folder`, `clear_folder`, `set_custom_artwork`,
`delete`, `list`, `get`, `get_tracks`, `add_tracks`, `reorder`,
`remove_track` at `crate::local_playlists::*` (i.e.
`qbz_library::local_playlists::*`) — this module is reached by
`LibraryDatabase::with_connection` per the module doc, and is almost
certainly called from Slint/Tauri command layers in other crates (search
for `local_playlists::` call sites outside this crate before finalizing).

## Coupling / watch out
- `init_schema` also creates `playlist_folders` (a table conceptually
  "owned" elsewhere — this module's doc says it's duplicated here only so
  standalone unit tests work; production already creates it first). Keep
  this whole CREATE TABLE batch string together in `schema.rs` — don't
  split the `execute_batch` call.
- The two additive-migration guards in `init_schema` (favorite/hidden,
  then folder_id) are independent of each other but each is internally a
  check-then-ALTER pair; keep each pair intact, order doesn't matter
  between the two guards themselves.
- `hydrate_counts` (in `playlist_ops.rs`) is called by both `list` and
  `get`, and reads from `local_playlist_tracks` (whose CRUD lives in
  `track_ops.rs`) — this is an intentional cross-file read; no shared
  mutable state, just a query, so no special handling needed beyond
  keeping both modules `pub(crate)`-visible to each other (default within
  the same crate).
- `add_tracks`' de-dup check and `reorder`'s position-shifting logic are
  each one non-trivial multi-statement SQL sequence — copy verbatim,
  don't rewrite the position arithmetic during the split.
- `folder_id` in `LocalPlaylist`/`move_to_folder`/`clear_folder` couples
  to the shared `playlist_folders` table used by Qobuz-side playlist
  folder org too — a cross-cutting concept spanning outside this file
  (relevant to whichever agent covers playlist folder code elsewhere).

## Verify after split
- `cargo test -p qbz-library local_playlists::` — all tests green (uses
  in-memory SQLite, no shared state, safe in parallel).
- `cargo check -p qbz-library` and any downstream crate consuming
  `qbz_library::local_playlists::*` (grep for the import path) to confirm
  nothing broke.
- Manual smoke-test: in the running app, create a local playlist, add a
  mix of Qobuz + local tracks, reorder, remove one, favorite/hide it, move
  it into a folder, delete it — confirms schema + both CRUD halves still
  interoperate correctly end to end.
