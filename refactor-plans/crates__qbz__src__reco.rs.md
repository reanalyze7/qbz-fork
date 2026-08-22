# crates/qbz/src/reco.rs (298 lines)

## Summary
Per-user `RecoStore` runtime wrapper: a process-global `Mutex<Option<
RecoStore>>` plus typed helpers for play/favorite/playlist-add event
logging, home-seed/surface reads, genre backfill, and async training — the
Slint app's equivalent of the Tauri recommendation event store.

## Proposed split
By responsibility, matching the file's own `// -----` section banners
(lifecycle, play events, favorite events, playlist-add events, surfaces,
training, tests):

- `reco/mod.rs` (~35 lines) — module doc, `pub mod` declarations, `pub
  use` re-exports of every public fn so `crate::reco::X` paths are
  unchanged.
- `reco/lifecycle.rs` (~25 lines) — `RECO` static, `init_for_user`,
  `teardown`.
- `reco/events.rs` (~110 lines) — `is_qobuz_source`, `log_play_gated`,
  `log_favorite_track`, `log_favorite_album`, `log_favorite_artist`,
  `insert_favorite`, `log_playlist_add` (all the write-path event
  loggers).
- `reco/surfaces.rs` (~80 lines) — `home_seeds`, `forgotten_favorite_
  album_ids`, `scored_favorite_album_ids`, `backfill_album_genres`,
  `known_artist_ids`, `recent_track_ids` (the read-path helpers consumed
  by Discover/Home rails).
- `reco/train.rs` (~15 lines) — `train_async`.
- `reco/tests.rs` (~25 lines) — the `#[cfg(test)] mod tests` block.

## Re-export surface
`reco/mod.rs` re-exports `init_for_user`, `teardown`, `is_qobuz_source`,
`log_play_gated`, `log_favorite_track`, `log_favorite_album`,
`log_favorite_artist`, `log_playlist_add`, `home_seeds`,
`forgotten_favorite_album_ids`, `scored_favorite_album_ids`,
`backfill_album_genres`, `known_artist_ids`, `recent_track_ids`,
`train_async` at `crate::reco::*` — this is called from `crate::auth`
(login/restore/logout call `reco::init_for_user`/`teardown`/`train_async`),
from playback code (`log_play_gated`), from favorite/playlist-add command
handlers, and from Discover/Home surface builders — a genuinely
wide fan-in, so the flat `crate::reco::fn_name()` call convention must be
preserved exactly (no new nesting like `crate::reco::events::log_play_
gated` at call sites).

## Coupling / watch out
- The single `static RECO: Mutex<Option<RecoStore>>` is the ONE piece of
  shared state every sub-module reaches into — each file just does
  `RECO.lock()` independently; no risk of split-induced bugs here since
  Rust statics are crate-visible regardless of which file declares them
  (declare it in `lifecycle.rs`, reference via `super::RECO` or
  `crate::reco::RECO` — actually needs `pub(super)` or similar visibility
  from `lifecycle.rs` so sibling files under `reco/` can reach it; since
  all are descendants of `reco/mod.rs` this works with the default
  private visibility as long as everything stays nested under `reco/`).
- `is_qobuz_source`'s exclusion list (`"local" | "ephemeral"`) is
  cross-referenced in a comment as "Same exclusion the mix seeder uses
  (`mix.rs`)" — if `mix.rs` (a different file, likely `myqbz_mix.rs` or a
  sibling `mix.rs`) has its own copy of this gate, flag to whichever agent
  covers that file that the two must stay in sync; not fixable within
  this file's split alone.
- `log_favorite_track` builds its `RecoEventInput` inline while
  `log_favorite_album`/`log_favorite_artist` route through the shared
  `insert_favorite` helper — a pre-existing minor inconsistency (not
  introduced by this split); keep `insert_favorite` in the same file as
  its two callers (`events.rs`) so the asymmetry stays visible/fixable in
  one place rather than scattered across a "helpers" file.
- `home_seeds` (surfaces.rs) is called internally by `scored_favorite_
  album_ids` (also surfaces.rs) — keep both in the same file, this is a
  same-file call today and should stay one after the split.
- `#[allow(dead_code)] recent_track_ids` — its doc comment says it's
  "currently unused... kept for the external-reco filters" — don't drop
  it as dead code during the split; it's intentionally kept for a
  near-future caller.

## Verify after split
- `cargo test -p qbz reco::` — both tests green (`helpers_are_noop_when_
  uninitialized` explicitly calls `teardown()` first, so it's order-
  sensitive only with itself, not with other tests — fine to keep as-is).
- `cargo check -p qbz` to confirm every `crate::reco::*` call site
  (auth.rs, playback event logging, favorite/playlist-add handlers,
  Discover/Home surface builders) still resolves.
- Manual smoke-test: log in, play a Qobuz track to completion (confirm no
  panic / a play event logs), favorite a track/album/artist, add tracks
  to a playlist, open a Discover rail that reads `home_seeds`/`scored_
  favorite_album_ids` and confirm it still renders, log out and back in
  to confirm `init_for_user`/`teardown` still cycle cleanly.
