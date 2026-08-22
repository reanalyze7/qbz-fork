# crates/qbz-app/src/settings/search_ranking.rs (486 lines)

## Summary
Per-query interaction ranking for Intelligent Search's "cortinilla" (inline
suggestion strip): a local, telemetry-free JSON-persisted `SearchRanking` store
that learns which entities a user opens/plays/favorites per normalized query,
with LRU query eviction and score capping — plus ~130 lines of unit tests.

## Proposed split
The file is already internally organized with `// ---- Section ----` banners
(tunables / interaction action / persisted shape / in-memory store / tests) —
follow those exactly, moving the pure-data and pure-logic pieces apart from the
tiny bit of IO (file read/write):

- `search_ranking/mod.rs` (~25 lines) — module doc (the big privacy/persistence/
  bounds/architecture-note comment block, lines 1-33), re-exports of
  `SearchRanking`, `InteractionAction`, and the tunable consts.
- `search_ranking/tunables.rs` (~20 lines) — `WEIGHT_OPEN`/`WEIGHT_PLAY`/
  `WEIGHT_FAVORITE`, `MAX_SCORE`, `MAX_QUERIES` (46-58).
- `search_ranking/action.rs` (~25 lines) — `InteractionAction` enum + `weight()`
  (66-85).
- `search_ranking/schema.rs` (~35 lines) — the on-disk shape: `ScoredEntity`,
  `QueryBucket`, `RankingDoc` (96-118).
- `search_ranking/store.rs` (~135 lines) — the `SearchRanking` struct + `new()`
  + `load()` + `persist()` (124-245): the IO half (file read/parse/write). This
  is right at the 130 line boundary — if it runs over, split `persist()`
  (serialization + write, ~55 lines) into `search_ranking/persist.rs` as a
  free fn taking `&SearchRanking` (or a `pub(super)` method extension via a
  second small `impl SearchRanking` block in that file).
- `search_ranking/ops.rs` (~115 lines) — the pure query/mutate operations:
  `touch`, `enforce_query_cap`, `record`, `top_for_query`, `score_for`,
  `rank_within` (249-352) as a second `impl SearchRanking` block. These are the
  "business logic" methods distinct from the load/persist IO.
- `search_ranking/tests.rs` (~130 lines) — the entire `#[cfg(test)] mod tests`
  block (358-486): weight accumulation, top_for_query max, score cap, LRU
  eviction, persistence round-trip, corrupt-file-loads-empty, rank_within
  stability. Kept as one file since these are integration-style store tests
  that exercise load+record+persist together.

## Re-export surface
`search_ranking/mod.rs` re-exports `SearchRanking`, `InteractionAction`, and the
weight/cap constants at `crate::settings::search_ranking::*` — the parent
`settings` module's existing `pub use search_ranking::...` (or `mod
search_ranking;` + direct path use) stays unchanged, so callers in the Slint
controller (`qbz-slint`'s search module, per the doc comment referencing "the
qbz-slint controller") keep working with zero changes.

## Coupling / watch out
- `normalize_query` is imported from a SIBLING module: `use
  super::search_cache::normalize_query;` (line 40) — this cross-file dependency
  on Capa A's cache module must be preserved exactly; don't accidentally break
  the `super::` path when nesting `search_ranking` one level deeper into its own
  subdirectory (path becomes `super::super::search_cache` if `search_ranking.rs`
  becomes `search_ranking/mod.rs` — actually a `mod.rs` keeps the SAME module
  depth as the original file, so `super::search_cache` still resolves; but any
  work split into `search_ranking/ops.rs` etc. needs `use
  super::super::search_cache::normalize_query` since those are one level deeper
  than `search_ranking/mod.rs`).
- `SearchRanking`'s `ranking: HashMap<String, HashMap<(String,String), i64>>`
  and `order: HashMap<String,u64>` are two separate maps that must stay in sync
  (every `ranking` key needs an `order` entry and vice versa) — `touch()`,
  `enforce_query_cap()`, and `record()` all maintain this invariant; keep them
  in the same file (`ops.rs`) so the invariant isn't spread across files.
- `persist()` and `load()` must use the EXACT same JSON shape
  (`RankingDoc`/`QueryBucket`/`ScoredEntity`) — if `schema.rs` changes
  independently of `store.rs`, round-trip tests will catch drift, but keep them
  logically paired in review even though physically split.
- This module is explicitly NOT allowed to depend on `QbzCore` or call
  `search_all` (per the architecture note) — don't introduce such a dependency
  while splitting.

## Verify after split
- `cargo build -p qbz-app`.
- `cargo test -p qbz-app search_ranking` — all 7 tests green, especially
  `persistence_round_trips` (load/persist IO) and
  `rank_within_reorders_scored_ahead_keeping_unscored_stable` (stable-sort
  correctness, which is easy to break with an off-by-one during refactor).
- `grep -rn "search_ranking::" crates/qbz-app crates/qbz-slint` (or wherever the
  Slint controller lives) to confirm the cortinilla-ranking call sites still
  resolve unchanged.
