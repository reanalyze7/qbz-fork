# crates/qbz-app/src/settings/album_play_history.rs (332 lines)

## Summary
"Most played albums" tracking (For You carousel + main.rs's Most-Played
row): a lazily-opened module-level SQLite connection (`db_path`,
`init_schema`, `open_db`, `with_db`), `AlbumPlayMeta`/`AlbumPlayRow`
structs, private `record_on`/`query_on` implementations, and the public
API (`record_album_play`, `top_albums`, `all_albums`). Has a `#[cfg(test)]`
module.

## Proposed split
- `album_play_history/mod.rs` (~15 lines) — re-exports.
- `album_play_history/db.rs` (~65 lines) — `db_path`, `init_schema`,
  `open_db`, `with_db` (lines 26-95) — the connection-management/schema
  layer.
- `album_play_history/model.rs` (~30 lines) — `AlbumPlayMeta<'a>`,
  `AlbumPlayRow` structs (lines 96-124).
- `album_play_history/queries.rs` (~90 lines) — `record_on`, `query_on`
  (lines 125-210), the actual SQL logic (upsert-on-replay, rank-by-count
  with recency tiebreak).
- `album_play_history/api.rs` (~30 lines) — the 3 public entry points:
  `record_album_play`, `top_albums`, `all_albums` (lines 211-237), which
  just call `with_db` + the queries module.
- `album_play_history/tests.rs` (~95 lines) — the `#[cfg(test)] mod tests`
  block (lines 238-332).

## Re-export surface
`album_play_history/mod.rs` re-exports `AlbumPlayMeta`, `AlbumPlayRow`,
`record_album_play`, `top_albums`, `all_albums` at their current
`qbz_app::settings::album_play_history::X` paths. Confirmed external
callers: `crates/qbz/src/foryou.rs` (`top_albums(20)`),
`crates/qbz/src/main.rs` (×4, `AlbumPlayRow` type + `all_albums()`), and
`crates/qbz/src/playback.rs` (`record_album_play(AlbumPlayMeta { ... })`)
— all four names must keep resolving at exactly this path.

## Coupling / watch out
- The module-level lazy `open_db()`/`with_db()` pattern (not a struct-
  based Store like sibling settings files) means there's no `Self` to
  thread through — the split is purely by responsibility layer, no state-
  visibility concerns like the struct-based files in this batch.
- `record_on`'s "upsert refreshes on replay" semantics (per the test
  `meta_upsert_refreshes_on_replay`) and `query_on`'s tie-break-by-recency
  ranking (`tie_break_prefers_more_recent_play`) are both behaviorally
  subtle — keep each function whole, don't split mid-function.
- `AlbumPlayMeta<'a>` borrows string slices — if `queries.rs` and
  `model.rs` are separate files, the lifetime parameter must stay
  consistent across both.

## Verify after split
- `cargo test -p qbz-app settings::album_play_history` green.
- `cargo build -p qbz` (three external call sites) and `-p qbz-app`.
