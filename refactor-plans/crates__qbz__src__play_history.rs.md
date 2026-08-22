# crates/qbz/src/play_history.rs (155 lines)

## Summary
Local SQLite-backed play history (`play_events` + `artist_names` tables) used
by the discovery pipeline's "skip artists I already know" filter: lazily opens
a WAL-mode SQLite DB, records a play per track-start, and answers "known
artists above a play-count threshold" as both a Qobuz-id set and a
normalized-name set.

## Proposed split
Only marginally over budget (155 lines); split DB bootstrap/connection
management from the two public query/write operations:

- `play_history/mod.rs` (~35 lines) — module doc, `DB` static, `db_path()`,
  `with_db` helper, `mod` declarations, `pub use record_play, known_artists`.
- `play_history/schema.rs` (~50 lines) — `open_db()` (connection open + WAL
  pragma + `CREATE TABLE IF NOT EXISTS` schema for `play_events` +
  `artist_names`).
- `play_history/queries.rs` (~75 lines) — `record_play` (the write path) and
  `known_artists` (the threshold-based read path), both `pub` API functions.

## Re-export surface
`play_history/mod.rs` re-exports `record_play` and `known_artists` — the only
two `pub` items in the original file (per the `#[allow(dead_code)]` comments,
consumed by `playback::record_recent` and `artist::load_mb_discovery`
respectively) — so those call sites (`crate::play_history::record_play(...)`,
`crate::play_history::known_artists(...)`) need no changes.

## Coupling / watch out
- `DB: OnceLock<Mutex<Option<Connection>>>` and `with_db` must stay in
  `mod.rs` since both `record_play` and `known_artists` (queries.rs) call
  `with_db(...)` — reference as `super::with_db` from `queries.rs`.
- `open_db()` (schema.rs) is called exactly once, lazily, from `with_db`'s
  `DB.get_or_init(|| Mutex::new(open_db()))` in `mod.rs` — needs
  `use super::schema::open_db;` (or keep `with_db` itself in `mod.rs` calling
  a `pub(super) fn open_db()` from `schema.rs`).
- `db_path()` is only used inside `open_db()` — could move entirely into
  `schema.rs` instead of `mod.rs` if preferred, shrinking `mod.rs` further;
  either placement is fine since both are in the same module.
- This file is small enough that a simpler alternative is: only extract
  `schema.rs` (open_db + db_path, ~65 lines) and leave `record_play` +
  `known_artists` + `with_db` + `DB` together in `mod.rs` (~90 lines) — a
  2-file split instead of 3. Either satisfies the 130-line rule; the 3-file
  version groups "read" vs "write" more cleanly for future growth.

## Verify after split
- `cargo check -p qbz` / `cargo build`.
- No existing unit tests in this file; none to keep green. The DB-path/schema
  logic is a reasonable candidate for a future `#[cfg(test)]` using an
  in-memory or tempdir SQLite connection, but not required by this task.
- Smoke-test: play a track, confirm no warnings logged for
  `play_history insert event failed` / `upsert name failed`, then trigger the
  discovery "skip known artists" filter and confirm previously-played artists
  are excluded.
