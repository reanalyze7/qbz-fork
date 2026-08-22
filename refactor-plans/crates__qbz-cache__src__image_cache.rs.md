# crates/qbz-cache/src/image_cache.rs (205 lines)

## Summary
Framework-agnostic LRU disk cache service (`ImageCacheService`) for Qobuz
album/artist images: SQLite-backed metadata (hash, url, size, last-accessed),
on-disk files keyed by MD5(url), with get/store/evict/stats/clear
operations. Shared verbatim by the Tauri app and the Slint shell.

## Proposed split
The struct's methods split cleanly along "open/schema" vs "read path" vs
"write/evict/clear path":

- `image_cache/mod.rs` (~20 lines) — module doc, `pub use` re-export of
  `ImageCacheStats`, `ImageCacheService`. Since `ImageCacheService`'s
  methods must live with its `impl` block (or be split via multiple `impl
  ImageCacheService` blocks across files, which Rust allows), this file can
  also just hold the struct definitions and constructor if the impls are
  fully moved out — see below.
- `image_cache/types.rs` (~20 lines) — `ImageCacheStats` struct,
  `ImageCacheService` struct definition (fields only).
- `image_cache/open.rs` (~40 lines) — `impl ImageCacheService { pub fn
  new() -> ... }` (cache-dir resolution, SQLite open, WAL pragma, schema
  creation) plus the private `url_hash`/`cache_path` helpers used by every
  other impl block.
- `image_cache/access.rs` (~35 lines) — `impl ImageCacheService { pub fn
  get(...); pub fn store(...); }` — the two per-image read/write paths.
- `image_cache/maintenance.rs` (~90 lines) — `impl ImageCacheService { pub
  fn evict(...); pub fn stats(...); pub fn clear(...); }` — the
  size-management operations.

Rust allows multiple `impl Type { ... }` blocks for the same struct across
different files in the same module, so this split needs no trait
indirection — each file just re-opens `impl ImageCacheService`.

Given the file is only ~1.6x over budget, a simpler 2-way split (types+open
vs access+maintenance) also satisfies the rule if preferred.

## Re-export surface
`image_cache/mod.rs` is the `mod image_cache;` target already used as
`qbz_cache::image_cache::{ImageCacheService, ImageCacheStats}` by both the
Tauri app and the Slint shell. Both names stay reachable via `pub use
types::*;` — the impl blocks in the other files don't need re-exporting
since they attach directly to the `ImageCacheService` type already
re-exported.

## Coupling / watch out
- `url_hash` and `cache_path` (private helpers defined in `open.rs` under
  this plan) are called from EVERY other impl block (`get`, `store`,
  `evict`'s per-entry path construction) — make them `pub(super)` or
  `pub(crate)` associated fns/methods so `access.rs` and `maintenance.rs`
  can call them across files; a private fn in one file's `impl` block is
  NOT visible from another file's `impl` block for the same type unless
  visibility is widened.
- `get`'s "file missing → clean up stale DB entry" self-healing behavior is
  a subtle correctness detail (keeps the DB and filesystem from drifting
  apart) — preserve it exactly, don't accidentally drop the cleanup
  `DELETE` when moving `get` into `access.rs`.
- `evict`'s LRU walk order (`ORDER BY last_accessed ASC`) and its
  `to_free`/`freed` bytes-accounting loop are one cohesive unit — don't
  split the query from the deletion loop.
- Both Tauri and Slint consumers share this exact cache directory
  (`~/.cache/qbz/images`) and DB file — purely an internal-module split, no
  behavior change, but worth calling out since any accidental behavior
  change here affects two frontends at once.

## What to verify after the real split
- `cargo build -p qbz-cache`.
- `cargo test -p qbz-cache` (no dedicated unit tests currently in this
  file — consider whether the split is a good moment to add a couple, per
  the project's "tests at each change" rule, e.g. a store→get roundtrip and
  an evict-under-limit test using a tempdir).
- Smoke-test importers: `cargo build -p qbz` (Slint shell) and the Tauri
  app crate both depend on `ImageCacheService` — confirm both still build
  and that image thumbnails still load/cache in a running instance of each.
