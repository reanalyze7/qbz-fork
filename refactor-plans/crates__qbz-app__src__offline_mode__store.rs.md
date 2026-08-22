# crates/qbz-app/src/offline_mode/store.rs (426 lines)

## Summary
Per-user `offline_settings.db` store (SQLite via rusqlite), byte-compatible
with Tauri's schema: settings (manual offline mode, network-folder policy,
the issue-#279 stream-first-track snapshot) plus a Last.fm scrobble queue
API. Roughly half the file (lines 297-426) is `#[cfg(test)]` tests.

## Proposed split
By responsibility (schema/init vs settings API vs scrobble-queue API vs
tests), which also naturally separates pure struct defs from I/O:

- `offline_mode/store/mod.rs` (~25 lines) — module doc, `pub use` of
  `OfflineModeSettings`, `QueuedScrobble`, `OfflineModeStore`; `mod tests;`.
- `offline_mode/store/types.rs` (~25 lines) — lines 23-44:
  `OfflineModeSettings`, `QueuedScrobble` struct defs (pure data, no logic).
- `offline_mode/store/schema.rs` (~75 lines) — lines 46-117: `OfflineModeStore`
  struct + `new_at()` (table creation SQL + the additive-migration list) —
  this is the "byte-compatible-with-Tauri" contract, kept as its own file so
  future migrations are added in one obvious place.
- `offline_mode/store/settings.rs` (~65 lines) — lines 119-179:
  `get_settings`, `set_manual_offline_mode`,
  `set_show_network_folders_in_manual_offline`,
  `get_pre_offline_stream_first_track`, `set_pre_offline_stream_first_track`
  — as `impl OfflineModeStore` methods (Rust allows splitting `impl` blocks
  across files in the same module tree via multiple `impl` blocks, one per
  file, all `impl OfflineModeStore` for the type defined in `schema.rs`).
- `offline_mode/store/scrobble_queue.rs` (~115 lines) — lines 187-295:
  `queue_scrobble`, `get_queued_scrobbles`, `mark_scrobbles_sent`,
  `cleanup_sent_scrobbles`, `queued_scrobble_count` — another `impl
  OfflineModeStore` block, the Last.fm-queue-specific half of the API.
- `offline_mode/store/tests.rs` (~130 lines) — lines 297-426: the existing
  `#[cfg(test)] mod tests` body verbatim (defaults, manual flag round-trip,
  network-folders round-trip, scrobble-queue round-trip, stream-first
  snapshot round-trip, Tauri-era-DB-reopen compatibility test).

## Re-export surface
`offline_mode/store/mod.rs` becomes the `mod store;` target already used as
`crate::offline_mode::store::{OfflineModeStore, OfflineModeSettings,
QueuedScrobble}` (or via `offline_mode::mod.rs`'s own re-export, if one
exists) — check whether `offline_mode/mod.rs` already does `pub use
store::*;`; if so no caller-visible path changes at all.

## Coupling / watch out
- Splitting `impl OfflineModeStore` across `settings.rs` and
  `scrobble_queue.rs` requires each file to `use super::schema::OfflineModeStore;`
  (or wherever the struct itself lands) — Rust permits multiple `impl` blocks
  for the same type across files in one crate, so this is safe, just make
  sure the struct's single field (`conn: Connection`) stays private and
  accessible to both impl-block files (same-module privacy, fine since
  they're siblings under `offline_mode/store/`).
- The migrations list in `schema.rs` (lines 102-111) is explicitly documented
  as matching Tauri's own list — do not reorder or renumber; new columns are
  always appended, never inserted.
- `tests.rs`'s `reopens_tauri_era_database_without_data_loss` test
  hand-constructs a pre-migration schema inline — this test doc-comments the
  exact reason schema changes must stay additive; keep this test colocated
  with (or at least referencing) `schema.rs`'s migration list.
- `unique_test_dir` helper in the test module uses a static `AtomicU64` nonce
  — if tests move to their own file, this helper and its `use
  std::sync::atomic::{AtomicU64, Ordering};` import must move together.

## Verify after split
- `cargo check -p qbz-app` and `cargo test -p qbz-app offline_mode::store` —
  all 6 existing tests must stay green, especially the Tauri-compat reopen
  test.
- Smoke-test: launch the app, toggle manual offline mode and the
  network-folders setting in the offline settings UI, confirm they persist
  across a restart.
