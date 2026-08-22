# crates/qbz-app/src/settings/reco_store.rs (1339 lines)

## Summary
Headless (Tauri-cleanroom-ported) recommendation event store: SQLite schema
(`reco_events` / `reco_scores` / `reco_album_meta`), event logging, ID-seed
read APIs for the Discover/home rows, a decay-weighted `train()` scorer, and
an extensive `#[cfg(test)]` suite — all behind the `RecoStore` struct.

## Proposed split
- `reco_store/mod.rs` (~90 lines) — the long module doc (lines 1-37), all
  small shared types (`RecoEventType`, `RecoItemType`, `RecoEventInput`,
  `TopArtistSeed`, `HomeSeeds`, `HomeSeedLimits` + its `Default`,
  `TrainParams` + its `Default`, `now_ts()`), `pub use` re-exports of
  `RecoStore`, `RecoStoreState`, `create_empty_reco_store_state`, and
  `#[cfg(test)] mod tests;` declaration.
- `reco_store/schema.rs` (~110 lines) — `RecoStore` struct definition,
  `open_at`, `new`, `new_at`, `init` (the big `execute_batch` DDL),
  `migrate_add_genre_id`. Owns DB lifecycle/migration only.
- `reco_store/events.rs` (~90 lines) — `insert_event`, `log_play_event`,
  `log_favorite_event` (the "write" half), plus the private
  `RecoEventRecord` struct and `get_events_since` (read helper `train()`
  needs) since they're tightly coupled to the event row shape.
- `reco_store/reads.rs` (~230 lines) — the plain read APIs:
  `get_recent_track_ids`, `get_recent_track_ids_since`,
  `get_favorite_track_ids`, `get_recent_album_ids`, `get_favorite_album_ids`,
  `get_top_artist_ids`, `get_known_artist_ids`, `get_top_genres`,
  `get_forgotten_favorite_album_ids`. These are all `impl RecoStore` methods
  in one `impl` block split across files via multiple `impl RecoStore { ... }`
  blocks (Rust allows this) — no re-declaration issue.
- `reco_store/scores.rs` (~130 lines) — the `reco_scores` companion-table
  methods: private `RecoScoreEntry` struct, `has_scores`,
  `get_scored_album_ids`, `get_scored_track_ids`, `get_scored_artist_scores`,
  `replace_scores`.
- `reco_store/home_seeds.rs` (~90 lines) — `get_home_seeds` (the
  scored/fallback merge logic) + `merge_unique_preserve_order` (currently a
  free fn at file scope, used only here).
- `reco_store/train.rs` (~130 lines) — the `train()` scorer method (decay
  factor, event/item weights, `build_scores`/`build_track_entries`/
  `build_album_entries`/`build_artist_entries` closures) — this is the
  single densest chunk (~120 lines as-is) and should stay intact as one
  method; do not further split the closures out since they capture `events`
  and `now` locally.
- `reco_store/meta.rs` (~35 lines) — `set_album_genre_name`,
  `update_genre_for_album` (the two `reco_album_meta` / genre-backfill
  writers) plus `RecoStoreState` type alias + `create_empty_reco_store_state`.
- `reco_store/tests.rs` (~220 lines) — the entire `#[cfg(test)] mod tests`
  block (lines 1120-1339): `unique_test_dir`, `insert_at`, and all 10 tests.

## Re-export surface
`reco_store/mod.rs` is the module root; existing callers use
`crate::settings::reco_store::{RecoStore, RecoStoreState,
create_empty_reco_store_state, RecoEventInput, RecoEventType, RecoItemType,
HomeSeeds, HomeSeedLimits, TrainParams, TopArtistSeed}` and all of these stay
importable unchanged via `mod.rs`'s re-exports plus `RecoStore`'s methods
being spread across multiple `impl RecoStore` blocks (transparent to callers
— Rust doesn't care which file an `impl` block lives in).

## Coupling / watch out
- `RecoStore` has exactly one field (`conn: Connection`) and every method
  across every proposed file is `impl RecoStore { ... }` — splitting the impl
  across files is mechanical (just repeat `impl RecoStore` in each file), but
  every file needs `use rusqlite::{params, Connection};` and the private
  helper types (`RecoEventRecord`, `RecoScoreEntry`) need `pub(super)` or
  `pub(crate)` visibility if they cross file boundaries (currently private to
  the single file) — e.g. `train.rs`'s closures build `RecoScoreEntry`
  values, so it needs to see the struct from `events.rs`/`scores.rs`.
- `train()` calls `get_events_since` (proposed in `events.rs`) and
  `replace_scores` (proposed in `scores.rs`) — both need `pub(super)` (or
  `pub(crate)`) visibility since they're currently private (`fn`, not
  `pub fn`) and are only ever called from within the same `impl RecoStore`.
- `get_home_seeds` calls `has_scores`, `get_scored_*` (scores.rs) and
  `get_recent_*`/`get_favorite_*`/`get_top_artist_ids` (reads.rs) — same
  private-fn-across-files visibility concern.
- The DB file path convention (`<base>/reco/events.db`, shared with Tauri) is
  documented in the top-of-file doc comment (lines 8-16) — keep that
  cross-reference intact in `mod.rs` since it's load-bearing context for
  anyone touching schema/migrations.
- Tests reach into `store.conn` directly (`insert_at` helper) — this needs
  `conn` to stay at least `pub(crate)` or the test helper needs to move
  inside the same module tree (it already will, as `reco_store/tests.rs`
  under `reco_store/`), so `pub(super)` on `conn` from `schema.rs` suffices.

## Verify after split
- `cargo test -p qbz-app settings::reco_store::` — all 10 existing tests
  green (schema idempotency, log/read, windowed query, top genres, home
  seeds fallback+trained, train ranking, forgotten favorites, genre backfill,
  known artists).
- `cargo check -p qbz-app` and grep for `reco_store::` importers (Discover
  seed callers, WeeklyQ) to confirm the public path is unchanged.
