# crates/qbz-offline-cache/src/path_validator.rs (236 lines)

## Summary
Validates a user-chosen offline cache root path (exists / is-a-directory /
mounted / writable), migrates cached FLAC files from an old root to a new
one when the user relocates the cache, and maintains a 30s-TTL mount-status
cache to avoid hammering `canonicalize()`/`metadata()` on every check.

## Proposed split
By domain — validation vs. file-move migration vs. the mount-status cache:

- `path_validator/mod.rs` (~15 lines) — module doc, `pub use` re-exports of
  `PathStatus`, `PathValidationResult`, `MoveReport`, `validate_path`,
  `check_permissions`, `check_mount_status`,
  `move_cached_files_to_new_path`, `is_offline_root_available`.
- `path_validator/validate.rs` (~80 lines) — `PathStatus` enum,
  `PathValidationResult` struct, `validate_path()`, `check_permissions()`
  (writes/removes a `.qbz_write_test` probe file), `check_mount_status()`
  (canonicalize + metadata check).
- `path_validator/migrate.rs` (~80 lines) — `MoveReport` struct,
  `move_cached_files_to_new_path()`, `collect_flac_files()` (recursive FLAC
  file walker) — the relocation flow.
- `path_validator/mount_cache.rs` (~45 lines) — `MountStatusCache` struct,
  the `static MOUNT_CACHE: Mutex<...>` + `CACHE_DURATION` const, and
  `is_offline_root_available()` — the 30s-TTL memoization wrapper around
  `check_mount_status`.

## Re-export surface
`path_validator/mod.rs` re-exports all six public items (`PathStatus`,
`PathValidationResult`, `MoveReport`, `validate_path`, `check_permissions`,
`check_mount_status`, `move_cached_files_to_new_path`,
`is_offline_root_available`) so
`crate::path_validator::{validate_path, move_cached_files_to_new_path,
is_offline_root_available, ...}` — used by the offline-cache settings UI's
"change storage location" flow — stays unchanged.

## Coupling / watch out
- `move_cached_files_to_new_path` calls `validate_path` (in `validate.rs`)
  on the NEW path before moving anything — needs
  `use super::validate::{validate_path, PathStatus};` after the split.
- `is_offline_root_available` (in `mount_cache.rs`) calls
  `check_mount_status` (in `validate.rs`) on cache miss/expiry — needs
  `use super::validate::check_mount_status;`.
- `check_mount_status` is called from BOTH `validate_path` (directly, every
  time) and `is_offline_root_available` (only on cache miss) — these are
  two different call patterns for the same underlying check; don't merge
  them, the doc/behavior difference (always-fresh vs 30s-cached) is
  intentional and callers pick whichever fits (validate_path for the
  one-shot "user just typed a path" check, is_offline_root_available for
  frequent polling e.g. before each playback).
- `MOUNT_CACHE` is a single global `static Mutex<Option<MountStatusCache>>`
  keyed by re-checking `cached.path == path` inside the lock — if the app
  ever validates two different offline roots concurrently (unlikely, single
  offline cache root per install) the cache only remembers the most recent
  path; this existing limitation is unchanged by the split, just flagging
  it's a global not a per-path map.
- `collect_flac_files` is recursive and only matches `.flac` extension —
  if the v2 CMAF format (tracks-cmaf directories, see `cmaf_store.rs` in
  this same batch) ever needs migrating too, this function would need
  updating; out of scope for this split but worth a comment noting v1-only.
- No `#[cfg(test)]` block exists in this file.

## Verify after split
- `cargo check -p qbz-offline-cache` / `cargo build -p qbz-offline-cache`.
- Grep `path_validator::` across the workspace (settings/offline-cache-
  location UI) to confirm every re-exported symbol's import path is
  unaffected.
- Manually exercise "change offline storage location" in the running app:
  point at an invalid path (expect the right `PathStatus` variant), then a
  valid one, and confirm existing cached FLACs actually move and the
  30s mount-cache doesn't produce stale "unmounted" results right after a
  drive is reconnected.
