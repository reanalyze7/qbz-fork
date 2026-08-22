# crates/qbz-external-reco/src/cache.rs (311 lines)

## Summary
Per-user SQLite (WAL) cache for resolved-recommendation -> Qobuz-id lookups,
with three logical tables/regimes: positive/negative id resolutions (30d/7d
TTL), built-result-row blobs (configurable TTL, default 48h), and resolved
weekly-playlist blobs (9d TTL + 21d stale-fallback) — plus a
`#[cfg(test)] mod tests` block (2 tests).

## Proposed split
By responsibility — schema/open vs each of the three cache regimes vs
tests. This is a moderate split since the whole file is one cohesive
`RecoCache` impl block; separate it by regime while keeping `RecoCache`
itself declared once:

- `cache/mod.rs` (~75 lines) — module doc, TTL constants
  (`FOUND_TTL_SECS`, `MISS_TTL_SECS`, `DEFAULT_RESULTS_TTL_SECS`,
  `WEEKLY_TTL_SECS`, `WEEKLY_STALE_FALLBACK_SECS`), `CacheLookup` enum,
  `RecoCache` struct definition, `impl RecoCache { fn open_at(...); fn
  now(); }` (schema creation + the shared `now()` clock helper), and `pub
  use` re-exports (none needed externally since `RecoCache`/`CacheLookup`
  stay declared right here).
- `cache/resolutions.rs` (~50 lines) — `impl RecoCache` block (as a
  second/third `impl RecoCache { ... }` block in a different file — Rust
  allows splitting one type's methods across files via multiple `impl`
  blocks) containing `get()` and `put()` (the positive/negative id-cache
  regime) and `cleanup_expired()`'s found/miss portion — OR keep
  `cleanup_expired()` whole in `mod.rs` since it touches all three tables
  (see coupling note below).
- `cache/results.rs` (~35 lines) — `get_results()`, `put_results()`,
  `clear_results()` (the built-result-blob regime).
- `cache/weekly.rs` (~60 lines) — `get_weekly()`, `put_weekly()`,
  `get_latest_weekly_for_patch()` (the weekly-playlist regime).
- `cache/mod.rs` also keeps `cleanup_expired()` (~15 lines) since it
  deletes from all three tables in one call — splitting it across files
  would mean three separate `impl RecoCache` methods calling into each
  other, which is messier than just leaving this one cross-cutting method
  where the constants and `RecoCache` struct already live.
- `cache/tests.rs` (~65 lines) — the existing `#[cfg(test)] mod tests`
  block (`tmp_dir` helper, `positive_negative_and_miss`,
  `weekly_cache_per_week_and_stale_fallback`), moved verbatim.

## Re-export surface
`cache/mod.rs` is the target of the existing `mod cache;` (or `pub mod
cache;`) declaration in `crates/qbz-external-reco/src/lib.rs`. `RecoCache`
and `CacheLookup` are declared in `mod.rs` and stay there; Rust's multiple-
`impl`-blocks-across-files pattern means every method (`open_at`, `get`,
`put`, `get_results`, `put_results`, `clear_results`, `get_weekly`,
`put_weekly`, `get_latest_weekly_for_patch`, `cleanup_expired`) remains
callable as `cache::RecoCache::method(...)` / `RecoCache::method(...)` with
NO change to any call site, since splitting `impl` blocks across files is
transparent to callers — this is one of the lower-risk splits in this
batch.

## Coupling / watch out
- `Self::now()` (in `mod.rs`) is called from EVERY method across every
  proposed submodule (`get`, `put`, `get_results`, `put_results`,
  `get_weekly`, `put_weekly`, `get_latest_weekly_for_patch`,
  `cleanup_expired`) — it must stay `pub(super)` or effectively visible via
  `impl RecoCache` (Rust methods on the same type are visible across
  `impl` blocks regardless of file, so this "just works" as long as
  `RecoCache`'s field `conn` stays private to the crate and each `impl
  RecoCache` block is declared with `use super::RecoCache;` — actually
  since these are `impl` blocks FOR `RecoCache`, they don't need field
  access beyond what's already `self.conn`, so no visibility issue at all).
- `cleanup_expired()` reads all 5 TTL constants declared in `mod.rs` — if
  it's kept in `mod.rs` (recommended above) this is a non-issue; if moved
  to a submodule, all 5 constants need to be visible from there (`pub(super)
  const` or `pub(crate) const`).
- The doc comment at the top explicitly notes the connection is `!Sync`
  and callers wrap it in a `Mutex` externally — this contract doesn't
  change with the split (still one `Connection` field on one struct), but
  worth flagging for whoever does the actual split so they don't
  accidentally introduce a second `Connection`/lock.
- Three independent SQL schemas (`reco_qobuz_cache`, `reco_results`,
  `reco_weekly`) are all created in one `execute_batch` inside `open_at()`
  — keep that whole batch statement together in `mod.rs`, don't split the
  `CREATE TABLE` calls across files even though the runtime methods that
  use each table get split.

## Verify after split
- `cargo test -p qbz-external-reco` — both existing tests
  (`positive_negative_and_miss`, `weekly_cache_per_week_and_stale_fallback`)
  must pass unchanged; they exercise all three regimes end-to-end
  (open_at → put/get → weekly put/get → stale-fallback), so they're a
  strong regression net for this split.
- `cargo check -p qbz-external-reco` to confirm downstream callers (the
  external-reco carousel builder that calls `RecoCache::open_at`,
  `.get`/`.put`, `.get_results`/`.put_results`, `.get_weekly`/`.put_weekly`)
  still compile.
- Smoke-test: trigger an External Recommendations rebuild in the running
  app (or via whatever CLI/test harness exists) and confirm cached rows
  still populate/expire correctly (positive/negative Qobuz-id resolution,
  48h results rotation, weekly playlist per-mbid caching).
