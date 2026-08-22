# crates/qbz-reco/src/builder.rs (405 lines)

## Summary
`ArtistVectorBuilder` — builds a combined artist vector from MusicBrainz
relationships + Qobuz similar artists, persists via `ArtistVectorStore`.
Careful `Arc<Mutex/RwLock>` locking pattern (locks dropped before `.await`)
because the MusicBrainz cache holds a `!Sync` rusqlite connection.

## Proposed split
By data source, since `build_vector` orchestrates two independent fetches:

- `builder/mod.rs` (~90 lines) — `ArtistVectorBuilder` struct, `BuildResult`,
  `new()`, `build_vector()` (the orchestration method — must stay together
  since it directly interleaves both sources' locking pattern).
- `builder/musicbrainz.rs` (~120 lines) — `build_from_musicbrainz`,
  `extract_relationships` (the raw-response → `ArtistRelationships` mapper).
- `builder/qobuz.rs` (~50 lines) — `build_from_qobuz`.
- `builder/ensure.rs` (~45 lines) — `ensure_vector` (freshness check + build).
- `builder/tests.rs` (~15 lines) — existing test module.

## Re-export surface
`builder/mod.rs` stays the `mod builder;` target with `ArtistVectorBuilder`
and `BuildResult` defined there; other files add `impl ArtistVectorBuilder`
blocks via `use super::ArtistVectorBuilder;`. `crate::builder::
ArtistVectorBuilder` path is unchanged for `suggestions.rs`.

## Coupling / watch out
- **Locking discipline is the critical thing to preserve**: every guard
  (`store.lock().await`, `mb_cache.lock()`, `qobuz_client.read().await`) is
  scoped in a block and dropped before crossing an `.await` — this is called
  out explicitly in the file's doc comment as required for the suggestions
  future to remain `Send`. When splitting into separate `impl` blocks/files,
  do NOT accidentally hold a lock across a call into another file's method.
- `extract_relationships` is a free fn (not a method) called only from
  `build_from_musicbrainz` — keep them in the same file (`musicbrainz.rs`).
- `mb_cache: Arc<std::sync::Mutex<...>>` (sync mutex, not tokio) vs
  `store`/`qobuz_client` (tokio Mutex/RwLock) — a std::sync guard must never
  be held across `.await`; this distinction must survive the split.

## Verify after split
- `cargo test -p qbz-reco builder::` (1 test: `test_weights_applied`).
- `cargo check -p qbz-reco` — confirm no `Send` regressions in async fns
  that get spawned (check `cargo build` catches any future-not-Send error).
- Smoke-test suggestions flow end-to-end if feasible (build_vector →
  suggestions engine) since that's where the Send requirement bites.
