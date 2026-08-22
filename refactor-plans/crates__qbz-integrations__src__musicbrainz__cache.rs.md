# crates/qbz-integrations/src/musicbrainz/cache.rs (724 lines)

## Summary
`MusicBrainzCache`: a single SQLite-backed struct providing TTL-expiring caches for
six different MusicBrainz entity kinds (recordings, artists, releases, artist
relations, artist metadata, scene discovery) plus a Qobuz-artist-validation cache and
a V2 "structured" resolved-track/resolved-artist cache, plus settings (enabled flag)
and maintenance (cleanup/clear/stats) — all as one large `impl` block with a
schema-init string.

## Proposed split
This is a clear by-domain split: each cache "kind" is already delimited by
`// ============ Section ============` banners with near-identical
get/set method pairs. Split into one file per cache domain, keeping the schema
definition centralized since SQLite `CREATE TABLE IF NOT EXISTS` statements for all
tables must run together at construction time.

- `cache/mod.rs` (~90 lines) — module doc, TTL constants (all 7 `*_TTL_SECS`),
  `CacheStats` struct, `MusicBrainzCache` struct definition, `new()`, `init_schema()`
  (the full multi-table `execute_batch` — keep as ONE statement since schema init is
  inherently a single atomic operation, not split further), `current_timestamp()`,
  `normalize_name()` (shared pure helper). Re-exports everything.
- `cache/settings.rs` (~30 lines) — `is_enabled()`, `set_enabled()` (a second `impl
  MusicBrainzCache` block).
- `cache/recording.rs` (~40 lines) — `get_recording()`, `set_recording()`.
- `cache/artist.rs` (~50 lines) — `get_artist_by_name()`, `set_artist_by_name()`
  (legacy JSON cache) — keep separate from V2 `get_artist`/`put_artist` below since
  they're genuinely different tables/formats.
- `cache/release.rs` (~40 lines) — `get_release()`, `set_release()`.
- `cache/relations.rs` (~45 lines) — `get_artist_relations()`, `set_artist_relations()`.
- `cache/metadata.rs` (~30 lines) — `get_artist_metadata()`, `set_artist_metadata()`.
- `cache/scene.rs` (~50 lines) — `get_scene_cache()`, `set_scene_cache()`.
- `cache/qobuz_validation.rs` (~30 lines) — `get_qobuz_validation()`,
  `set_qobuz_validation()`.
- `cache/resolved_v2.rs` (~130 lines) — `get_track()`, `put_track()`, `get_artist()`,
  `put_artist()` — the structured V2 cache (both track and artist, since they share
  the `MatchConfidence`/`increment_stat` pattern and are the newer format described
  together in the file).
- `cache/maintenance.rs` (~100 lines) — `cleanup_expired()`, `clear_all()`,
  `get_stats()`, `cleanup()`, `increment_stat()` (private helper used across V2
  methods — see coupling note).

## Re-export surface
`cache/mod.rs` re-exports `MusicBrainzCache` and `CacheStats` — all external callers
use `crate::musicbrainz::cache::{MusicBrainzCache, CacheStats}` exactly as before.
Every method stays a `pub fn` on `MusicBrainzCache` via additional `impl` blocks
across the split files (Rust allows this within the same crate).

## Coupling / watch out
- `increment_stat()` (private helper) is called from `get_track`/`get_artist` in
  `resolved_v2.rs` but is proposed to live in `maintenance.rs` — either move it to
  `resolved_v2.rs` instead (its only callers) or make it `pub(super)` so
  `resolved_v2.rs` can call `super::maintenance::increment_stat` — simplest fix:
  colocate `increment_stat` with `resolved_v2.rs`, not `maintenance.rs`.
- `current_timestamp()` (mod.rs) is used by nearly every get/set method across all
  domain files — keep it `pub(super)` or `pub(crate)` in `mod.rs`.
- `normalize_name()` is used by both `artist.rs` (legacy) and `resolved_v2.rs`
  (`get_artist`/`put_artist` use `.to_lowercase()` directly, NOT `normalize_name` —
  note this inconsistency already exists in the current file; preserve it exactly,
  don't "fix" it as part of a pure refactor).
- `init_schema()`'s single `execute_batch` call creates ALL 9 tables in one string —
  do NOT split this into per-domain schema strings run separately, since a partial
  Cargo split-driven change to run them as N separate `execute_batch` calls changes
  transaction/error semantics subtly (multi-statement `execute_batch` treats the
  whole string as one operation for error reporting purposes) — keep it as the one
  large SQL string in `mod.rs`, just note table ownership per domain in comments.
- `conn: Connection` (single shared `rusqlite::Connection`, not `Arc`/`Mutex`) is used
  by every method across every split file — no special handling needed since they're
  all still `&self` methods on the same struct, just verify no file accidentally
  takes `&mut self` where others expect `&self` (all current methods are `&self`).

## Verify after split
- `cargo check -p qbz-integrations`
- `cargo test -p qbz-integrations musicbrainz::cache` (check whether tests exist
  elsewhere in the crate, e.g. `tests/` integration tests, that exercise this cache —
  none are in this file itself).
- Grep for `MusicBrainzCache::` construction/usage across `qbz-integrations` and any
  crate depending on it (likely the MusicBrainz enrichment service) to confirm the
  public API surface (`new`, `is_enabled`, all get/set/cleanup methods) is unchanged.
