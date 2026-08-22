# crates/qbz/src/scrobble.rs (776 lines)

## Summary
Slint-side scrobbler controller: Last.fm + ListenBrainz auth flows bound to
Slint callbacks, plus the source-agnostic now-playing/scrobble firing engine
(arm-on-track-change, offline queueing into shared per-user SQLite stores,
and a queue-flush watcher on the offline-mode engine).

## Proposed split
By domain (auth UI vs. fire/schedule vs. offline flush), each a submodule of
a new `scrobble/` directory. `mod.rs` keeps the long module doc, shared
statics, and `start()`.

- `scrobble/mod.rs` (~90 lines) — module doc (lines 1-29), `set_status`
  helper, `LASTFM_PENDING_TOKEN` / `RT_HANDLE` / `FLUSH_WATCHER` statics,
  `rt_handle()`, `start()`, `seed_listenbrainz_from_shared_cache()`, and
  `pub use` re-exports of the other submodules' public items.
- `scrobble/panel.rs` (~110 lines) — panel init/toggle callbacks: `load`,
  `enable_toggle`, `collapse_toggle`.
- `scrobble/lastfm_auth.rs` (~125 lines) — `lastfm_enable_toggle`,
  `lastfm_connect`, `lastfm_open_auth_url`, `lastfm_confirm`,
  `lastfm_disconnect` (the two-step OAuth-like flow around
  `LASTFM_PENDING_TOKEN`).
- `scrobble/listenbrainz_auth.rs` (~85 lines) — `listenbrainz_enable_toggle`,
  `listenbrainz_set_token`, `listenbrainz_disconnect`.
- `scrobble/fire.rs` (~130 lines) — `ScrobbleMeta`, `SCROBBLE_GEN`,
  `on_track_changed`, `lb_info`, `send_now_playing`, `send_scrobble` (the
  now-playing + delayed-scrobble firing engine).
- `scrobble/queue.rs` (~65 lines) — `queue_lastfm`, `queue_listenbrainz`,
  `listenbrainz_cache_path` (writing a failed/offline fire into the shared
  queues).
- `scrobble/flush.rs` (~130 lines) — `flush_offline_queues`,
  `flush_lastfm_queue`, `flush_listenbrainz_queue` (draining both queues on
  reconnect / shell entry).

## Re-export surface
`scrobble/mod.rs` re-exports every `pub fn` (`start`, `load`,
`enable_toggle`, `collapse_toggle`, `lastfm_*`, `listenbrainz_*`,
`on_track_changed`, `ScrobbleMeta`) at `crate::scrobble::*` via `pub use`, so
`main.rs`'s callback bindings and the playback-poll call site
(`crate::scrobble::on_track_changed`) need no changes.

## Coupling / watch out
- `listenbrainz_cache_path()` is used by `mod.rs` (seeding),
  `listenbrainz_auth.rs` (write-through/clear on disconnect), and
  `queue.rs`/`flush.rs` (queue + flush) — keep it in `mod.rs` (or a small
  shared `paths.rs`) and have all four submodules `use super::listenbrainz_cache_path;`.
- `rt_handle()` is read by `fire.rs` (`on_track_changed` spawns from it);
  keep it in `mod.rs` alongside the `RT_HANDLE` static it wraps.
- `SCROBBLE_GEN` (monotonic generation guard) is read+written only inside
  `fire.rs` — no cross-file coupling risk there.
- `scrobbler_settings::get()` / `ScrobblerSettings` (a different file,
  `crate::scrobbler_settings`) is read from nearly every submodule — no
  change needed, just note it's an external dependency, not something to
  fold into this split.
- The Hi-Res/gold-badge note in the task brief does not apply to this file
  (no Slint UI badge code lives here) — nothing to preserve specially.

## Verify after split
- `cargo check -p qbz` (or the crate `scrobble.rs` lives in) compiles.
- `cargo test -p qbz scrobble::` if/when tests are added (currently this
  file has none — do not invent new coverage during the split, only
  preserve existing behavior).
- Grep for `crate::scrobble::` and `scrobble::on_track_changed` importers in
  `main.rs`/`playback.rs` to confirm the public path is unchanged.
