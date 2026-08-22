# crates/qbzd/src/scrobble_engine.rs (325 lines)

## Summary
Daemon background task: subscribes to the CoreEvent bus, sends now-playing +
scrobbles to Last.fm/ListenBrainz on the play-half-or-4-min rule, and runs a
periodic ListenBrainz offline-queue drain (persisted via `ListenBrainzCache`).

## Proposed split
- `mod.rs` (~60 lines) — `Playing` struct, `DRAIN_INTERVAL` const, and the
  public `spawn()` entry point (the `tokio::spawn` event loop / `select!`).
- `providers.rs` (~50 lines) — `now_playing`, `scrobble`, `lb_client`
  (per-provider now-playing/scrobble dispatch to Last.fm/ListenBrainz).
- `queue.rs` (~90 lines) — `queue_listenbrainz`, `drain_listenbrainz`,
  `lb_cache_path` (the offline-queue persistence/retry logic).
- `pure.rs` (~20 lines) — `due`, `album_opt`, `now_unix` (pure helpers, no
  I/O — easy standalone unit tests).
- `tests.rs` (~45 lines) — existing `#[cfg(test)] mod tests`, split to test
  `pure.rs`'s functions (`due`, `album_opt`).

## Re-export surface
`mod.rs` stays the only public surface: `pub fn spawn(...)`. Nothing else in
this file is called from outside the module (verify with `grep -rn
scrobble_engine:: crates/qbzd/src`).

## Coupling / watch-outs
- `Playing` (with its `threshold`/`scrobbled` fields) is read/written across
  the event loop (`mod.rs`) — keep it there, not in `providers.rs`.
- `queue.rs`'s `drain_listenbrainz` and `mod.rs`'s `spawn` both need
  `roots: &ProfileRoots` and `s: &ScrobblerSettings` threaded through — no
  shared mutable state beyond function args, so this is a low-risk split.
- The rusqlite `Connection` (`ListenBrainzCache::new`) must stay opened
  inside `spawn_blocking` in whichever file owns `queue_listenbrainz`/
  `drain_listenbrainz` — do not accidentally hold it across an `.await`.

## Verify after split
`cargo build -p qbzd`; `cargo test -p qbzd scrobble_engine`; smoke-test by
running `qbzd` and confirming a scrobble is sent/queued as before.
