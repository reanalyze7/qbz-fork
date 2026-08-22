# crates/qbz/src/playback.rs (4530 lines)

## Summary
The Slint frontend's playback/queue orchestration layer: turns albums,
artists, playlists, and ad-hoc track lists into `Vec<QueueTrack>` handed to
`QbzCore`'s `QueueManager`, drives audible playback through `Player`, and
runs the single 450ms poll loop that is the ONLY source of playback events
(no event stream from the player) — driving the now-playing bar, MPRIS/tray/
notifications, gapless prefetch, seamless-transition reconciliation,
watchdog recovery, session-position persistence, and auto-advance.

## Scale note
This is by far the largest file in the sweep (~35x over budget). Given its
size, this plan identifies major responsibility clusters (per the task
brief) rather than a line-by-line module map. Each cluster below is sized to
land under 130 lines AFTER extraction, but the poll loop itself (~530 lines)
will need a second decomposition pass once the sub-steps are pulled into
named functions — see the dedicated "poll loop" section.

## Proposed split (by responsibility cluster)

### 0. Shared state (`playback/state.rs`, ~60 lines)
All the process-wide statics currently scattered through the file, in one
place since they're read/written from multiple clusters below:
`QUEUE_CONTROLLER`, `PREFETCH_SEMAPHORE`, `PENDING_PLAY_ID`/
`PENDING_PLAY_AT_MS`, `UNAVAILABLE_SKIPS`, `WATCHDOG_RECOVER_TRACK`/
`WATCHDOG_RECOVERIES`, `NOTIFY_LAST_TRACK`, `MPRIS_LAST_META`,
`FORCE_UI_REPUSH`, `TRACK_MAX_RATE_HZ`/`TRACK_MAX_BITS`,
`REQUESTED_QUALITY_ID`/`REQUESTED_CAUSE`, `HYDRATED_TRACK_ID`/
`HYDRATED_RATE_HZ`/`HYDRATED_BITS`, `MUTED`/`PREMUTE_VOLUME`. Plus
`set_queue_controller`/`refresh_sidebar` (the two functions that gate on
`QUEUE_CONTROLLER`). All must be `pub(super)` or `pub(crate)` so every other
new module can reach them.

### 1. Audible-playback engine (`playback/engine.rs`, ~120 lines)
`after_track_change`, `play_audible`, `kick_prefetch` — the core
"fetch bytes -> hand to Player -> update UI" pipeline shared by every play
path. `PREFETCH_SEMAPHORE` usage stays here.

### 2. Advance / auto-skip / offline gating (`playback/advance.rs`, ~120 lines)
`advance_to_playable`, `try_infinite_refill` (documented dead branch —
keep the doc comment explaining why), `auto_skip_unavailable`,
`local_track_file_exists`, `offline_playability`, `offline_track_playable`,
`is_terminal_unavailable`, `is_forbidden_backoff`. All the "can we actually
play this track right now" logic used by both manual play and auto-advance.

### 3. Loading-spinner + watchdog (`playback/loading.rs`, ~60 lines)
`set_loading`, `clear_loading` — small but conceptually separate from the
watchdog RECOVERY logic in the poll loop, which reads/writes the same
`PENDING_PLAY_ID`/`PENDING_PLAY_AT_MS` statics.

### 4. Local file & ephemeral playback (`playback/local.rs`, ~230 lines)
`play_local_file_audible`, `play_local_tracks_now`, `play_local_album`,
`play_ephemeral_all`, `play_ephemeral_album`, `play_ephemeral_track`,
`ephemeral_play`, `ephemeral_enqueue`, `ephemeral_play_or_prompt`,
`play_local_tracks`, `play_local_folder_recursive`,
`play_local_folder_tracks_from`, `play_local_tracks_from`,
`fill_missing_covers`. Likely needs a further split into `local/ephemeral.rs`
(the `play_ephemeral_*`/`ephemeral_*` family) and `local/files.rs` (the
`play_local_*`/`fill_missing_covers` family) to land under 130 each.

### 5. Now-playing metadata sync (`playback/meta.rs`, ~280 lines)
`refresh_now_playing_meta` (the ~450-line function is the single largest in
the file — title/artist/album/artwork/quality-badge/context/MPRIS/tray/
notification), `load_now_playing_artwork`, `load_now_playing_artwork_large`,
`mpris_meta_changed`, `set_now_playing_context`, `hydrated_catalog_quality`.
`refresh_now_playing_meta` alone will need internal extraction into 2-3
helper functions (e.g. `build_meta_fields(&QueueTrack) -> MetaFields`,
`push_meta_to_ui(...)`, `sync_mpris_and_tray(...)`) before this file lands
under 130 lines — treat this as its own mini-file-split
(`meta/build.rs`, `meta/push.rs`, `meta/mpris_tray.rs`).

### 6. Quality/format helpers (`playback/quality.rs`, ~90 lines)
`fmt_elapsed`, `fmt_remaining`, `now_ms`, `set_viz_paused`,
`recent_quality`, `album_card_meta`. Small pure formatters currently
interleaved with the metadata cluster — worth its own file since they're
called from both `meta.rs` and the poll loop.

### 7. Recently-played + blacklist (`playback/recent_blacklist.rs`, ~90 lines)
`record_recent`, `queue_track_blacklisted`, `filter_blacklisted_queue`,
`track_is_blacklisted_full`.

### 8. Album/artist/label context resolution (`playback/context_play.rs`, ~280 lines)
`fetch_album_for_play`, `play_album`, `play_album_from`,
`fetch_artist_top_for_play`, `play_artist_top_tracks`, `play_artist`,
`enqueue_artist_top_selected`, `play_artist_top_shuffled`,
`play_label_top_shuffled`, `play_artist_top_from`, `make_top_track_queue`.
Split further into `context_play/album.rs` and `context_play/artist.rs` if
needed (artist-related functions are the bulk, ~180 lines alone).

### 9. Track/queue building (`playback/queue_build.rs`, ~200 lines)
`play_track_now`, `mmss_to_secs`, `track_item_to_queue`, `model_ids`,
`reorder_queue_by_visible`, `queue_from_model`, `order_by_visible`,
`play_queue`, `play_track_in_context`, `play_tracks`, `play_tracks_ctx`.
`play_track_in_context` alone is ~175 lines and is the most likely
candidate for its own file if this cluster doesn't fit under 130.

### 10. Enqueue commands (`playback/enqueue.rs`, ~330 lines)
`enqueue_album`, `play_album_shuffled`, `enqueue_album_next`,
`enqueue_track`, `play_track_next`, `play_playlist`, `enqueue_playlist`,
`enqueue_track_ids`, `enqueue_tracks`, `enqueue_queue_tracks`,
`enqueue_local_tracks`. All thin Slint-callback-shaped wrappers around the
queue-building/engine clusters above — split into `enqueue/album.rs`,
`enqueue/track.rs`, `enqueue/playlist.rs` if still over budget.

### 11. Transport controls (`playback/transport.rs`, ~190 lines)
`toggle_play_pause`, `next`, `previous`, `seek`, `set_volume`,
`toggle_mute` (owns `MUTED`/`PREMUTE_VOLUME`), `toggle_shuffle`,
`cycle_repeat`.

### 12. Tests (`playback/tests.rs`, `#[cfg(test)] mod tests` — size TBD from
current ~30-line module, likely stays as-is or grows as helpers move).

### 13. The poll loop (`playback/poll/mod.rs` + sub-files, ~530 lines total)
`start_poll_loop` is one giant `loop { ticker.tick().await; ... }` body that
cannot be split across files as-is (a single `loop` can't span modules).
The split strategy is: extract each documented phase — already clearly
commented with `// --- Phase name ---` banners in the source — into a named
async function taking `&mut PollLoopState` (a new struct holding
`last_track_id`/`was_playing`/`seen_position`/`save_pos_tick`/
`gapless_requested_for`/`last_ui_push`), called in sequence from the thin
loop body left in `poll/mod.rs`:
- `poll/mod.rs` (~60 lines) — `start_poll_loop`, the `PollLoopState` struct,
  the bare `loop { tick; call each phase in order }`.
- `poll/stream_errors.rs` (~35 lines) — the stream-error-to-toast surfacing
  phase.
- `poll/seamless.rs` (~70 lines) — the gapless seamless-transition detection
  + queue-pointer reconciliation phase.
- `poll/gapless_prefetch.rs` (~130 lines) — the gapless-prefetch-trigger
  phase (network track branch + local/DSD track branch); likely needs
  `gapless_prefetch/network.rs` + `gapless_prefetch/local.rs` given its
  current ~170-line size in source.
- `poll/ui_push.rs` (~90 lines) — the dirty-guarded `NowPlayingState` push
  (quality-downgrade classification + elapsed/remaining formatting).
- `poll/watchdog.rs` (~90 lines) — the stuck-load recovery phase
  (`WATCHDOG_RECOVER_TRACK`/`WATCHDOG_RECOVERIES` handling).
- `poll/track_end.rs` (~90 lines) — the end-of-track / stop-after /
  auto-advance / queue-finished phase.

## Re-export surface
`playback/mod.rs` (the new top-level, replacing the old flat `playback.rs`)
re-exports every currently-`pub` item unchanged: `set_queue_controller`,
`after_track_change`, `play_local_album`, `wipe_ephemeral_if_playing`,
`play_ephemeral_*`, `ephemeral_*`, `play_local_tracks*`,
`play_local_folder_*`, `fill_missing_covers`, `set_now_playing_context`,
`play_album*`, `play_artist*`, `enqueue_artist_top_selected`,
`play_label_top_shuffled`, `play_track_now`, `play_track_in_context`,
`play_tracks*`, `enqueue_album*`, `enqueue_track*`, `play_track_next`,
`play_playlist`, `enqueue_playlist`, `enqueue_tracks*`,
`enqueue_local_tracks`, `toggle_play_pause`, `next`, `previous`, `seek`,
`set_volume`, `toggle_mute`, `toggle_shuffle`, `cycle_repeat`,
`start_poll_loop`. Every caller currently does `crate::playback::foo(...)`
from Slint callback wiring elsewhere in the crate — none of those call
sites should need to change import paths if `mod.rs` re-exports flatly via
`pub use {engine::*, advance::*, local::*, meta::*, context_play::*,
queue_build::*, enqueue::*, transport::*, poll::start_poll_loop, ...};`.

## Tricky coupling / watch out for the actual split
- **The statics are the biggest hazard.** Nearly every cluster reads or
  writes at least one static from `playback/state.rs`. Do this split
  incrementally, one cluster at a time, running `cargo check` after each
  move — do NOT attempt a single big-bang split of all 14 clusters at once.
- **`refresh_now_playing_meta`, the poll loop, and `after_track_change` form
  a tight triangle**: the poll loop calls `refresh_now_playing_meta` on a
  seamless transition, `after_track_change` calls it (indirectly, check),
  and both read/write `MPRIS_LAST_META`/`NOTIFY_LAST_TRACK`/
  `FORCE_UI_REPUSH`. Keep the dedupe-guard semantics (each guard fires
  exactly once per real change) intact across the split — this is the
  highest-risk area for a silent regression (stale now-playing bar, or
  MPRIS spam).
- **`gapless_requested_for` is loop-local state**, not a static — when
  extracting the gapless-prefetch phase into `poll/gapless_prefetch.rs`, it
  must be threaded through as `&mut PollLoopState`, not accidentally
  promoted to a static (which would break its per-loop-invocation reset
  semantics, though in practice there's only ever one loop instance via the
  `STARTED` guard).
- **The watchdog and end-of-track phases both reset `last_track_id`/
  `was_playing`/`seen_position`/`gapless_requested_for`** on different
  triggers (stop-after vs. queue-finished vs. successful advance) — audit
  every reset site carefully when moving them into separate files so none
  is dropped or duplicated.
- **`try_infinite_refill` is a documented dead branch** (qbz-radio was
  removed, it always returns `false` now) — preserve the doc comment
  explaining this so a future agent doesn't "helpfully" delete the
  now-dead-looking call site in the track-end handler, which is
  intentionally left as the single fallback-chain exit point.
- **`hydrated_catalog_quality`/`TRACK_MAX_RATE_HZ`/`TRACK_MAX_BITS`/
  `REQUESTED_QUALITY_ID`/`REQUESTED_CAUSE`** feed the poll loop's
  quality-downgrade badge logic (`stream_downgraded`, `classify_limit_cause`,
  `delivered_tier_str` — these three helper functions live OUTSIDE the
  1000-4530 range shown here and may be in a different file/module; grep for
  them before finalizing `poll/ui_push.rs`'s imports).
- **Queue-controller sidebar refresh** (`refresh_sidebar`) is called from
  almost every cluster (engine, advance, poll loop, enqueue commands) — it's
  a thin wrapper already isolated in `playback/state.rs`'s proposed home;
  confirm no cluster ends up needing a local re-declaration.

## What to verify after the real split
- `cargo check -p qbz` after EVERY incremental cluster move (not just at
  the end) — this file is too large and too central to playback for a
  single "split everything then check" pass to be safe.
- `cargo test -p qbz playback` (the existing `#[cfg(test)] mod tests` at
  line ~3968, currently small — check what it actually covers before/after).
- Manual smoke test covering: play an album, play a single track, enqueue
  next, gapless transition between two tracks (network), gapless transition
  for local/DSD files, seek, volume/mute, shuffle/repeat, pause mid-track
  then resume, let a track play to completion (auto-advance), stop-after-
  this-song, offline mode with an unavailable track in the queue (auto-skip),
  MPRIS controls (play/pause/next/prev from a system tray or OS media
  widget), and the loading-watchdog path (hardest to trigger manually —
  consider a deliberately-corrupt local file to force a stuck load).
- Confirm the tray tooltip and desktop media-control integration
  (`crate::tray`, `crate::media_controls`) still receive play/pause/track
  updates — these are easy to silently disconnect during the poll-loop
  split since they're triggered on an edge (`is_playing != was_playing`)
  buried inside the loop body.
