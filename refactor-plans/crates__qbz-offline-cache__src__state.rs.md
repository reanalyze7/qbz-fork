# crates/qbz-offline-cache/src/state.rs (158 lines)

## Summary
`OfflineCacheState`: the plain (non-Tauri) struct holding the offline-cache's
open SQLite index, stream fetcher, cache-dir path, size limit, download
concurrency semaphore, and a separate library-DB connection for download
post-processing; both the Tauri and Slint frontends own one instance.

## Proposed split
Only 28 lines over budget — a light split, not a fragmentation exercise.

- `state/mod.rs` (~30 lines) — the `OfflineCacheState` struct definition +
  field docs; `pub use` / re-declares `init`/`init_at`/`teardown`/paths as
  `impl` blocks split across the sibling files below (Rust allows multiple
  `impl OfflineCacheState` blocks in different files of the same module).
- `state/lifecycle.rs` (~75 lines) — `new`, `new_empty`, `init_at`,
  `init_library_connection`, `teardown`: everything that opens/closes the
  DB(s) and creates directories.
- `state/paths.rs` (~35 lines) — `track_file_path`, `artwork_path`,
  `get_cache_path`, `apply_persisted_limit`: pure path/limit accessors that
  don't touch the DB.

## Re-export surface
`state/mod.rs` re-exports `OfflineCacheState` unchanged; callers already use
`crate::state::OfflineCacheState` (or `qbz_offline_cache::state::...`), so no
import paths change. Multiple `impl OfflineCacheState` blocks across
`lifecycle.rs`/`paths.rs` need `use super::OfflineCacheState;` — this is
valid Rust (impls do not need to be co-located with the struct) as long as
all files are declared as `mod` children in `state/mod.rs` (or the crate
converts `state.rs` into `state/` directory with `mod.rs`).

## Coupling / watch out
- `Arc<Mutex<...>>`/`Arc<RwLock<...>>` fields are cloned/shared with the
  Tauri and Slint frontends — do not change field visibility or types.
- `library_db` teardown order matters (closed before the main `db` — see
  `teardown`'s comment); keep that ordering when moved to `lifecycle.rs`.
- The 5 GB default limit constant appears in both `new` and `new_empty` —
  consider a single `const DEFAULT_LIMIT_BYTES` in `mod.rs` shared by both.

## Verify after split
- `cargo check -p qbz-offline-cache`
- `cargo test -p qbz-offline-cache`
- Grep for `qbz_offline_cache::state::OfflineCacheState` importers in
  `qbz-app`/`qbz` (Slint) and any Tauri `src-tauri` crate still present;
  confirm they still compile against the same path.
