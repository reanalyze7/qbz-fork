# crates/qbz-app/src/settings/playback.rs (452 lines)

## Summary
SQLite-backed playback preferences (autoplay mode, show-context-icon,
persist-session, resume-playback-position): schema/migration, a `Store` wrapping
one `Connection`, a process-level `State` wrapping `Arc<Mutex<Option<Store>>>` for
lifecycle (init/teardown across login sessions), plus ~120 lines of `#[cfg(test)]`.

## Proposed split
By pure/IO/test layering — this file is already well-organized internally, the
split just needs to separate the pieces into files.

- `settings/playback/mod.rs` (~30 lines) — module doc, `pub use` re-exports of
  `AutoplayMode`, `PlaybackPreferences`, `PlaybackPreferencesStore`,
  `PlaybackPreferencesState` from the split files below.
- `settings/playback/types.rs` (~75 lines) — `AutoplayMode` enum + its
  `Default`/`to_db_value`/`from_db_value`, `PlaybackPreferences` struct + its
  `Default`. Pure data + pure mapping, no I/O.
- `settings/playback/store.rs` (~165 lines) — `PlaybackPreferencesStore`: schema
  creation, the three `ALTER TABLE` migrations, `new`/`new_at`/`open_at`,
  `get_preferences`, all the `set_*` methods, `reset_all`, and the free
  `column_exists` helper. The SQLite I/O layer.
- `settings/playback/state.rs` (~85 lines) — `PlaybackPreferencesState`
  (`Arc<Mutex<Option<Store>>>` wrapper): `new`/`new_empty`/`init_at`/`teardown`
  and its own `get_preferences`/`set_*` passthroughs. The per-session lifecycle
  layer.
- `settings/playback/tests.rs` (~120 lines) — the entire `#[cfg(test)] mod
  tests` block, using `super::store::PlaybackPreferencesStore` and
  `super::types::{AutoplayMode, PlaybackPreferences}` (or simply `use
  super::*;` against `mod.rs`'s re-exports).

## Re-export surface
`settings/playback/mod.rs` is the public API: `pub use types::{AutoplayMode,
PlaybackPreferences}; pub use store::PlaybackPreferencesStore; pub use
state::PlaybackPreferencesState;`. Callers elsewhere in qbz-app / qbz that do
`qbz_app::settings::playback::PlaybackPreferencesState` (or however the crate
re-exports it) need zero changes.

## Coupling / watch out
- `PlaybackPreferencesStore` (store.rs) is used BY `PlaybackPreferencesState`
  (state.rs) as its inner type — state.rs needs `use super::store::
  PlaybackPreferencesStore;`, and store.rs must expose `new_at` as
  `pub(crate)` or `pub` (currently plain `pub fn`, keep it that way since
  state.rs is a sibling module, not a child).
- `AutoplayMode::to_db_value`/`from_db_value` (types.rs) are called from
  `store.rs` in `get_preferences`/`set_autoplay_mode`/`reset_all` — keep these
  as `pub(crate)` or accessible-enough methods on the enum (they're currently
  private `fn`s on the impl — fine to keep private if `store.rs` is a sibling
  submodule that can see `pub(super)`/crate-visible items; verify visibility
  after the split, may need to bump to `pub(crate)`).
- Tests reference `PlaybackPreferencesStore::new_at` directly (bypassing
  `PlaybackPreferencesState`) — after the split, `tests.rs` needs `use
  super::store::PlaybackPreferencesStore;` (or via `super::*` if `mod.rs`
  re-exports it, which it does).
- The doc comment at the top flags `show_context_icon` as a portable UI
  preference living here only for "settings contract" compatibility — keep
  that doc comment attached wherever `show_context_icon`'s schema/logic ends up
  (store.rs), not lost in the split.

## Verify after split
- `cargo build -p qbz-app`.
- `cargo test -p qbz-app playback` (or the equivalent test filter) — all 5
  existing tests (`playback_preferences_default_values_are_stable`,
  `_store_returns_defaults`, `_persist_all_fields`, `_migrates_legacy_schema`,
  `_reset_all_preserves_existing_behavior`) green.
- Smoke-test: `grep -rn "settings::playback::" crates/qbz-app/src
  crates/qbz/src` (or wherever it's consumed) still resolves.
