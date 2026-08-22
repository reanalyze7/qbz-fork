# crates/qbz-integrations/src/listenbrainz/cache.rs (288 lines)

SQLite-backed offline-listen cache for ListenBrainz: credentials, enabled
flag, listen queue CRUD.

## Proposed split

- `listenbrainz/cache/mod.rs` (~60 lines) — re-export surface,
  `ListenBrainzCache` struct + `new`/`init_schema` (schema owns the
  `CREATE TABLE` batch — keep with the constructor).
- `listenbrainz/cache/credentials.rs` (~50 lines) — `save_credentials`,
  `get_credentials`, `clear_credentials`, `is_enabled`, `set_enabled`
  (as `impl ListenBrainzCache` blocks in a separate file via
  `impl ListenBrainzCache { ... }` split across files — standard Rust
  pattern, no trait needed).
- `listenbrainz/cache/queue.rs` (~150 lines) — `queue_listen`,
  `get_pending_listens`, `mark_sent`, `increment_attempts`,
  `mark_listens_sent`, `get_queue_count`, `clear_queue`, `cleanup_sent`.

## Tricky coupling

- All three files `impl ListenBrainzCache` against the same struct defined
  in `mod.rs` — needs `use super::ListenBrainzCache;` and the struct's
  `conn` field must stay `pub(super)` (or keep impls in the same file as
  the struct if visibility is awkward — Rust allows multiple `impl` blocks
  across files as long as they're in the same crate and the type is
  visible).

## Verify after split

`cargo build -p qbz-integrations`, `cargo test -p qbz-integrations
listenbrainz::`.
