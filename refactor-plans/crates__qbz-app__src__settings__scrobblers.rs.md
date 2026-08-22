# crates/qbz-app/src/settings/scrobblers.rs (462 lines)

## Summary
Per-user SQLite-backed store for scrobbler settings (master enable + Last.fm
session/username + ListenBrainz token/username), a per-user
`ScrobblerSettingsState` lifecycle wrapper mirroring the app's other settings
stores, plus a full test suite.

## Proposed split
Same shape as the already-planned `pinned_items.rs` split (see
`refactor-plans/crates__qbz-app__src__settings__pinned_items.rs.md` for the
sibling convention): schema struct vs. store/lifecycle vs. Last.fm ops vs.
ListenBrainz ops vs. state wrapper vs. tests.

- `scrobblers/mod.rs` (~65 lines) — module doc (the long header comment,
  lines 1-28), `ScrobblerSettings` struct + its 4 helper methods
  (`lastfm_is_authed`, `listenbrainz_is_authed`, `lastfm_active`,
  `listenbrainz_active`), `pub use` of `ScrobblerSettingsStore` and
  `ScrobblerSettingsState` from the other files.
- `scrobblers/store.rs` (~50 lines) — `ScrobblerSettingsStore` struct +
  `open_at` (schema creation/migration) + `new_at` — construction/lifecycle
  only.
- `scrobblers/lastfm_ops.rs` (~40 lines) — `impl ScrobblerSettingsStore`
  continued: `get_settings` (shared read — could instead live in `store.rs`
  since it reads ALL fields, not just Last.fm; recommend keeping
  `get_settings` in `store.rs` and moving only `set_enabled`,
  `set_ui_collapsed`, `set_lastfm_enabled`, `set_lastfm_session`,
  `disconnect_lastfm` here).
- `scrobblers/listenbrainz_ops.rs` (~35 lines) — `set_listenbrainz_enabled`,
  `set_listenbrainz_token`, `disconnect_listenbrainz`.
- `scrobblers/state.rs` (~90 lines) — `ScrobblerSettingsState` struct +
  `Default` + `new_empty`/`init_at`/`teardown`/`with_store` (private) + all
  the thin `pub fn` forwarders (`get_settings`, `set_enabled`, ...,
  `disconnect_listenbrainz`) that delegate through `with_store`.
- `scrobblers/tests.rs` (~120 lines) — the entire `#[cfg(test)] mod tests`
  block (lines 343-462: `unique_test_dir` helper + all 5 tests), declared
  via `#[cfg(test)] mod tests;` in `mod.rs`.

## Re-export surface
`scrobblers/mod.rs` re-exports `ScrobblerSettings`, `ScrobblerSettingsStore`,
`ScrobblerSettingsState` at `crate::settings::scrobblers::*` — the Slint
`scrobble` controller mentioned in the module doc (which reads/writes through
`ScrobblerSettingsState` and separately writes ListenBrainz creds into the
shared `ListenBrainzCache.credentials` row) keeps importing from the same
path unchanged.

## Coupling / watch out
- `get_settings` reads ALL 8 columns in one query (both Last.fm and
  ListenBrainz fields) — do NOT try to split it into
  `get_lastfm_settings`/`get_listenbrainz_settings`, that would double the
  SQL round-trips; keep it as one method in `store.rs` even though it's
  logically "shared" rather than Last.fm- or ListenBrainz-specific.
  `lastfm_ops.rs`/`listenbrainz_ops.rs` each call `self.conn.execute(...)`
  directly (no shared private helper today), so no coupling risk in moving
  them — each method is fully self-contained SQL.
- The module doc (lines 1-28) explicitly cross-references the Last.fm
  offline queue (`scrobble_queue` table in `OfflineModeStore`, a DIFFERENT
  file/crate) and the ListenBrainz offline queue
  (`ListenBrainzCache.listen_queue` in `qbz-integrations`) — neither lives in
  this file, so nothing to relocate, but keep that doc comment intact in
  `scrobblers/mod.rs` since it's the map other engineers use to find the
  actual queue tables.
- `ScrobblerSettingsState.store: Arc<Mutex<Option<ScrobblerSettingsStore>>>`
  is the shared mutable state — defined once in `state.rs`, all forwarder
  methods there already go through the private `with_store` helper, so no
  duplication risk.
- Tests directly construct `ScrobblerSettingsStore` (bypassing
  `ScrobblerSettingsState`) for 3 of the 5 tests — when `tests.rs` is split
  out, it needs `use super::*;` reaching both `store::ScrobblerSettingsStore`
  and `state::ScrobblerSettingsState` (both re-exported from `mod.rs`, so
  `use super::*;` should resolve them fine since `mod.rs` re-exports them at
  the `scrobblers` module root).

## Verify after split
- `cargo test -p qbz-app scrobblers::` — all 5 tests
  (`scrobbler_settings_default_is_unconfigured`,
  `scrobbler_store_returns_defaults`, `scrobbler_persists_all_fields`,
  `scrobbler_disconnect_keeps_enable_flags`, `scrobbler_state_requires_init`)
  green.
- `cargo check -p qbz-app` and grep for `settings::scrobblers::` /
  `ScrobblerSettingsState` importers (the Slint `scrobble` controller) to
  confirm the public path is unchanged.
