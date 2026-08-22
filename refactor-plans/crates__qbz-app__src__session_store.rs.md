# crates/qbz-app/src/session_store.rs (661 lines)

## Summary
SQLite-backed `SessionStore`: persists the playback queue/session (current
track, position, volume, shuffle/repeat) plus Tauri/Svelte shell view-restore
fields, with schema migrations (`ALTER TABLE ... ADD COLUMN` guards) run at
open.

## Proposed split
Clean split by responsibility: data types vs. schema/lifecycle vs. read/write
ops vs. tests.

- `session_store/mod.rs` (~35 lines) — module doc, `pub mod` declarations,
  `pub use` re-exports of every public type/fn so `qbz_app::session_store::X`
  paths are unchanged.
- `session_store/model.rs` (~95 lines) — `PersistedQueueTrack`,
  `PersistedPlaybackSession` + its `Default` impl, `PersistedShellViewState` +
  `Default` impl + `default_last_view`/`default_streamable` helpers,
  `PersistedSessionSnapshot`.
- `session_store/schema.rs` (~155 lines) — `SessionStore` struct definition +
  `new`, `new_at`, `open_at` (the CREATE TABLE + all five `has_*`/`ALTER TABLE`
  migration guards — hires, is_local, source, streamable, last_view).
- `session_store/ops.rs` (~130 lines) — the read/write `impl SessionStore`
  block: `save_session`, `load_session` (the two biggest, transaction-wrapped
  methods) — likely needs splitting further into `ops/save.rs` (~85 lines,
  `save_session`) and `ops/load.rs` (~85 lines, `load_session`) since each is
  already 60-90 lines on its own.
- `session_store/quick_ops.rs` (~60 lines) — the smaller single-field savers:
  `save_position`, `save_volume`, `save_playback_mode`, `clear_session`.
- `session_store/pragma.rs` (~15 lines) — the `#[cfg(test)]`-only
  `pragma_synchronous`/`pragma_journal_mode` helpers (kept separate since
  they're test-only introspection, not part of the public read/write API).
- `session_store/tests.rs` (~165 lines) — the entire `#[cfg(test)] mod tests`
  block (`unique_test_dir`, `sample_track`, and the 5 tests:
  `default_session_values_are_stable`, `session_store_uses_wal_and_full_synchronous`,
  `session_store_round_trips_queue_and_shell_view_state`,
  `quick_saves_update_only_targeted_playback_fields`,
  `clear_session_resets_playback_and_shell_view_fields`).

## Re-export surface
`session_store/mod.rs` re-exports `PersistedQueueTrack`,
`PersistedPlaybackSession`, `PersistedShellViewState`,
`PersistedSessionSnapshot`, `SessionStore` at `crate::session_store::*` (i.e.
`qbz_app::session_store::*`) — this is a leaf module in the `qbz_app` crate
consumed by the `qbz` frontend crate, so the external path
`qbz_app::session_store::SessionStore` must stay valid.

## Coupling / watch out
- `conn: Connection` (rusqlite, not `Send`-shared beyond the struct) is
  defined once in `schema.rs`'s struct def; `ops.rs`/`quick_ops.rs`/
  `pragma.rs` just add more `impl SessionStore { ... }` blocks referencing
  `self.conn` — Rust allows multiple `impl` blocks for the same type across
  files in the same crate, no special handling needed.
- The five schema-migration guards in `open_at` are ORDER-DEPENDENT (each
  checks a column added by a *later* guard doesn't already exist, e.g.
  `has_hires` before `has_is_local` before `has_source` before
  `has_streamable`) — keep them in the same sequential order within
  `schema.rs`, don't reorder during the split.
- `save_session` wraps a manual `BEGIN TRANSACTION`/`COMMIT`/`ROLLBACK` (not
  rusqlite's `Transaction` type) — if `ops.rs` is split into `save.rs`/
  `load.rs`, keep the whole transaction block (including all three
  early-return-on-error branches) together in `save.rs`; don't split
  mid-transaction.
- Field lists in `save_session`'s INSERT and `load_session`'s SELECT are
  positional (17 columns, matched by index) — any accidental reordering
  during the split would silently break column mapping; copy these blocks
  verbatim.

## Verify after split
- `cargo test -p qbz-app session_store::` — all 5 tests green (they use
  temp dirs, no shared state, safe to run in parallel).
- `cargo check -p qbz-app` and `cargo check -p qbz` (or full workspace) to
  confirm the `qbz_app::session_store::SessionStore` import path used by the
  `qbz` frontend crate still resolves.
- Manual smoke-test: launch the app, play a track, quit, relaunch, and
  confirm the queue/position/volume/shuffle/repeat and last-view restore
  correctly (exercises both `save_session`/`load_session` and the quick-save
  paths together).
