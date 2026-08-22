# crates/qbz-library/src/qobuz_playlist_snapshot.rs (306 lines)

## Summary
Pure-SQLite (no Tauri/async) local snapshot of the user's Qobuz playlists
for offline mode: a names producer (`upsert_names`, cheap, every list-load)
and a membership producer (`replace_tracks`, full-replace, on detail-load),
plus read accessors (`get_header`, `all_headers`, `track_ids`,
`all_track_ids`) and schema init.

## Proposed split
By schema/read/write, a natural split for a small self-contained SQLite
module:

- `qobuz_playlist_snapshot/mod.rs` (~30 lines) — module doc (verbatim from
  the current header), `pub use` re-exports of `SnapshotHeader`,
  `SnapshotNameEntry`, `init_schema`, `upsert_names`, `replace_tracks`,
  `get_header`, `all_headers`, `track_ids`, `all_track_ids`, plus the shared
  `now_ms` helper.
- `qobuz_playlist_snapshot/schema.rs` (~25 lines) — `init_schema` (the two
  `CREATE TABLE IF NOT EXISTS` + index statements) and `now_ms`.
- `qobuz_playlist_snapshot/types.rs` (~20 lines) — `SnapshotHeader`,
  `SnapshotNameEntry` structs.
- `qobuz_playlist_snapshot/write.rs` (~75 lines) — `upsert_names`,
  `replace_tracks` (the two producers).
- `qobuz_playlist_snapshot/read.rs` (~70 lines) — `row_to_header`,
  `get_header`, `all_headers`, `track_ids`, `all_track_ids`.
- `qobuz_playlist_snapshot/tests.rs` (~90 lines) — the whole `#[cfg(test)]
  mod tests` block unchanged (roundtrip_header_and_tracks,
  replace_is_full_replace, names_only_rows_without_tracks,
  replace_refuses_unknown_playlist), `use super::*;`.

Given the file is only moderately over budget (306 lines, ~2.4x), this
5-file split is more than enough — a simpler 3-way split (types+schema,
read, write+tests) would also satisfy the 130-line rule if the reviewer
prefers fewer files.

## Re-export surface
`qobuz_playlist_snapshot/mod.rs` is the `mod qobuz_playlist_snapshot;`
target already used as `qbz_library::qobuz_playlist_snapshot::X` (or however
this crate re-exports it — check `qbz-library/src/lib.rs`). All current
public items stay reachable via `pub use schema::init_schema; pub use
types::*; pub use write::*; pub use read::*;`.

## Coupling / watch out
- `init_schema` is called from `LibraryDatabase::open` per the file's own
  doc comment ("run by `LibraryDatabase::open` next to the rest of the
  schema") — this is an OUTBOUND dependency from elsewhere in
  `qbz-library` into this module; verify that call site (likely in
  `qbz-library/src/lib.rs` or a `database.rs`) after the split still
  resolves `qobuz_playlist_snapshot::init_schema`.
- `replace_tracks`'s "refuse unknown playlist" behavior (returns `Ok(false)`
  without writing when there's no header row) is the load-bearing safety
  rule from the module doc ("a merely-viewed public playlist never lands in
  the snapshot") — keep this check literally inside `replace_tracks`, not
  hoisted into a generic helper that might get reused carelessly elsewhere.
- All functions take `&Connection` directly (no Tauri state, no async) —
  this is explicitly called out as enabling in-memory-SQLite testing; don't
  let the split introduce any state/wrapper type that would break that.

## What to verify after the real split
- `cargo build -p qbz-library`.
- `cargo test -p qbz-library qobuz_playlist_snapshot::` — all 4 existing
  tests green.
- Smoke-test importers: grep for `qobuz_playlist_snapshot::` call sites in
  `crates/qbz` (the names-producer call on playlist list-load, the
  membership-producer call on playlist detail-load) and confirm they still
  compile against the new module paths.
