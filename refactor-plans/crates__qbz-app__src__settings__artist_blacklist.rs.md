# crates/qbz-app/src/settings/artist_blacklist.rs (667 lines)

## Summary
Headless `BlacklistService` (frontend-agnostic port of Tauri's
`BlacklistService`): SQLite-backed artist + album blacklist with in-memory
`HashSet` caches for O(1) lookup, plus a shared enable/disable feature flag;
two parallel axes (artists by `u64` id, albums by `String` id).

## Proposed split
By axis + concern (schema/lifecycle vs. artist ops vs. album ops vs. tests),
matching the file's own "Album axis" comment boundary:

- `artist_blacklist/mod.rs` (~75 lines) — module doc, imports, `DB_FILE_NAME`
  const, `BlacklistedArtist`, `BlacklistedAlbum`, `BlacklistSettings` +
  `Default` impl, the `BlacklistService` struct definition, re-exports.
- `artist_blacklist/lifecycle.rs` (~100 lines) — `impl BlacklistService`
  block with `new`, `new_in_memory`, `init_schema`, `load_from_db`,
  `load_albums_from_db`, `load_settings` (construction + schema + initial
  load, shared by both axes).
- `artist_blacklist/artists.rs` (~110 lines) — `impl BlacklistService` block
  with `is_blacklisted`, `add`, `remove`, `get_all`, `count`, `set_enabled`,
  `is_enabled`, `get_settings`, `clear_all` (the artist axis + the shared
  enabled-flag accessors, since `set_enabled`/`is_enabled`/`get_settings`
  gate both axes but live naturally with the "first" axis).
- `artist_blacklist/albums.rs` (~110 lines) — `impl BlacklistService` block
  with `is_album_blacklisted`, `add_album`, `remove_album`, `get_all_albums`,
  `album_count`, `clear_all_albums` (mirrors `artists.rs` 1:1, minus the
  shared enabled-flag methods).
- `artist_blacklist/tests.rs` (~150 lines) — the entire `#[cfg(test)] mod
  tests`, referenced from `mod.rs` via `#[cfg(test)] mod tests;`, using
  `use super::*;`.

## Re-export surface
`artist_blacklist/mod.rs` — becomes
`crates/qbz-app/src/settings/artist_blacklist/mod.rs`. It keeps
`pub struct BlacklistService`, `pub struct BlacklistedArtist`,
`pub struct BlacklistedAlbum`, `pub struct BlacklistSettings`,
`pub const DB_FILE_NAME`. Since all the split-out methods are `impl
BlacklistService` blocks (Rust allows multiple `impl` blocks for the same
type across files/modules as long as they're in the same crate), no `pub use`
gymnastics are needed for the methods themselves — callers keep writing
`blacklist_service.add(...)`, `blacklist_service.is_album_blacklisted(...)`,
etc. unchanged. Only the struct/type re-exports at `mod.rs` matter.

## Coupling / watch out
- `enabled: AtomicBool` and `set_enabled`/`is_enabled` are shared by BOTH
  axes (`is_blacklisted` and `is_album_blacklisted` both short-circuit on
  it) — keep `set_enabled`/`is_enabled`/`get_settings` in one place
  (`artists.rs` per the plan above) and make sure `albums.rs` only ever
  *reads* `self.enabled`, never redefines it.
  - Consider a short doc-comment cross-reference at the top of `albums.rs`
    pointing to where `set_enabled` lives, so a future split doesn't
    duplicate the flag.
- `conn: Connection` (single `rusqlite::Connection`, not pooled) is shared by
  every method across all three per-axis impl blocks — no locking beyond
  Rust's normal `&self`/`&mut self` borrow rules; nothing to change here, but
  note `BlacklistService` is NOT `Sync`-safe for concurrent writes without an
  external mutex at the call site (same as today — not introduced by the
  split).
- Test module's `shared_enabled_flag_gates_both_axes` and
  `axes_are_independent` tests explicitly cross both axes — keep them in one
  `tests.rs` (don't split tests by axis) so these cross-cutting invariants
  stay easy to find and run together.
- `init_schema` creates BOTH tables (`artist_blacklist` and
  `album_blacklist`) in one `execute_batch` — don't split the schema DDL
  itself even though the split separates artist/album *operations*; keep
  `init_schema` whole in `lifecycle.rs`.

## Verify after split
- `cargo test -p qbz-app settings::artist_blacklist` — all 13 existing tests
  green (artist axis: 6, album axis: 4, shared: 2, plus roundtrip tests).
- `cargo check -p qbz-app` for any crate/module using
  `qbz_app::settings::artist_blacklist::{BlacklistService, DB_FILE_NAME,
  BlacklistedArtist, BlacklistedAlbum, BlacklistSettings}`.
- Smoke-test: the running app's artist/album blacklist UI (add/remove/toggle
  enabled) still works against an EXISTING `artist_blacklist.db` file (the
  schema/pragma must stay byte-identical — verify `init_schema`'s SQL text is
  copied verbatim, not paraphrased, during the split).
