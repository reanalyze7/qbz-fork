# crates/qbz-player/src/player/mod.rs (~5,649 lines)

**NOTE:** at this size, exact per-line planning is not practical. This plan was
built by reading the file's doc comments, `grep`-ing every top-level `fn`/`struct`/
`enum`/`impl`, and reading representative spans (lines 1-500, 1354-1650) rather than
every line end-to-end. Treat the line ranges below as approximate boundaries found
via `grep -n -E '^\s*(pub )?fn |^impl |^(pub )?(struct|enum) '`.

## Summary
The whole audio-playback engine: the `AudioCommand` protocol, a dedicated audio
thread (device init, decode, gapless/DSD/streaming playback, loudness
normalization), the `SharedState` cross-thread status block, and the public
`Player` API (`play_track`, `play_streaming`, `pause`/`seek`/`stop`, DSD file
playback, prefetch/cache, gapless handoff).

## Major responsibility clusters found (line ranges)

1. **Data types / small trait impls** (1-508, 608-924, 5375-5441): `AudioCommand`
   enum, `GaplessPending`, `CursorMediaSource` (+ `MediaSource`/`Read`/`Seek` impls),
   `AudioSpecs`, `AudioMetadata`, `StreamType` (+ impl), `CachedNominalRate`,
   `StreamRecreateDecision`, `PlaybackEvent`, `PlaybackState`, `DsdErrorReport`,
   `external_content_type`.
2. **Device / stream setup helpers** (174-924, free functions, not in `impl
   Player`): `cpal_device_name`, `decode_with_symphonia`, `is_isomp4`,
   `extract_audio_metadata[_full]`, `cached_quality_below_requested`,
   `decode_with_fallback`, `create_output_stream_with_config`,
   `apply_engine_volume`, `uses_coreaudio_system_default`,
   `query_macos_nominal_rate`, `coreaudio_nominal_rate`,
   `coreaudio_shared_rate_mismatch`, `evaluate_stream_recreate`,
   `compute_needs_new_stream`, `try_init_stream_with_backend`.
3. **`SharedState`** (925-1326): `PlaybackEvent` + `SharedState` struct/impl —
   ~40 small getter/setter methods (dsd mode, stream error, quality, gain,
   buffer progress, device, gapless flags, position/timer, playing/volume).
4. **`Player::new` / the audio thread** (1354-4057, ~2,700 lines): a single
   `thread::spawn` closure containing setup (loudness cache, `wrap_source`
   closure, `init_device` closure, ~1360-1633) followed by the command receive
   loop, matched by `AudioCommand` variant:
   - `Play` (1634-2126, ~490 lines)
   - `PlayStreaming` (2127-2680, ~550 lines)
   - `PlayDsdDop` (2681-2787), `PlayDsdNative` (2788-2905),
     `PlayNextDsdDop` (2906-3011) — DSD variants, ~100-120 lines each
   - `Pause` (3012-3025), `Resume` (3026-3177, ~150 lines — device
     re-init-on-resume logic), `Stop` (3178-3217), `SetVolume` (3218-3230),
     `Seek` (3231-3473, ~240 lines)
   - `ReinitDevice` (3474-3513), `ReleaseDevice` (3514-3537)
   - `PlayNext` (3538-4055, ~520 lines — gapless append)
5. **`Player` public API — command senders + async fetch** (4058-5374, ~1,300
   lines): `begin_play`/`is_current_play`, `play_track` (async),
   `prefetch_into_cache` (async), `is_track_cached`, `clear_audio_cache`,
   `fetch_for_gapless` (async), `fetch_for_external_stream` (async),
   `cmaf_stream_segments` (async), `play_data`/`apply_play_data`, `play_next`,
   `play_dsd_file`, `prepare_dsd_gapless_wav`, `play_next_dsd`,
   `is_dsd_direct_active`, `play_streaming`/`apply_play_streaming`,
   `play_streaming_dynamic`/`apply_play_streaming_dynamic`, `download_audio`,
   `pause`/`resume`/`stop`/`set_volume`/`seek`/`reinit_device`/
   `release_device`/`reload_settings`/`get_state`/`get_playback_event`.
6. **Tests** (5442-5649): unit tests for DSD error reporting, stream-recreate
   decision logic, position anchoring, content-type fallback, quality
   normalization.

## Proposed module split

Because cluster 4 is one giant closure sharing many captured locals (settings,
loudness cache, analyzer channels, `wrap_source`/`init_device` inline closures),
splitting it into separate files requires first extracting a `ThreadCtx` struct
that bundles the captured state, then turning each match arm into a
`fn handle_x(ctx: &mut ThreadCtx, ...)` in its own file. This is a real
refactor, not a pure move — flag it clearly for whoever does the actual split.

- `player/mod.rs` (~50 lines) — `mod` declarations + `pub use` re-exports of
  `Player`, `SharedState`, `PlaybackEvent`, `PlaybackState`,
  `external_content_type`; keeps `mod playback_engine; mod streaming_source;`.
- `player/types.rs` (~230 lines) — cluster 1's data types (may need one more
  split if it lands over 130; consider `types/audio.rs` for `AudioSpecs`/
  `AudioMetadata`/`CursorMediaSource` and `types/command.rs` for `AudioCommand`/
  `GaplessPending`/`StreamType`).
- `player/device/probe.rs` (~130 lines) — `cpal_device_name`,
  `uses_coreaudio_system_default`, `query_macos_nominal_rate`,
  `coreaudio_nominal_rate`, `coreaudio_shared_rate_mismatch`.
- `player/device/stream_init.rs` (~130 lines) — `create_output_stream_with_config`,
  `apply_engine_volume`, `try_init_stream_with_backend`.
- `player/device/recreate.rs` (~120 lines) — `evaluate_stream_recreate`,
  `compute_needs_new_stream`, `CachedNominalRate`, `StreamRecreateDecision`.
- `player/decode.rs` (~130 lines) — `decode_with_symphonia`, `is_isomp4`,
  `extract_audio_metadata[_full]`, `cached_quality_below_requested`,
  `decode_with_fallback`.
- `player/shared_state.rs` (~120 lines) — `PlaybackEvent` + most of
  `SharedState`.
- `player/shared_state_timer.rs` (~60 lines) — position/timer-related
  `SharedState` methods (`current_position[_ms]`, `start/pause_playback_timer`)
  if splitting keeps `shared_state.rs` under budget.
- `player/audio_thread/ctx.rs` (~130 lines) — new `ThreadCtx` struct + the
  `wrap_source`/`init_device` setup logic extracted from lines ~1360-1633.
- `player/audio_thread/mod.rs` (~80 lines) — `Player::new`'s `thread::spawn`
  body reduced to: build `ThreadCtx`, loop over `rx.recv()`, dispatch to
  `commands::handle_*`.
- `player/audio_thread/commands/play.rs` (~120 lines, may need `play_streaming.rs`
  split out separately since PlayStreaming alone is ~550 lines) — `Play` handler.
- `player/audio_thread/commands/play_streaming.rs` (split into 2 files if it
  stays >130 after extraction, e.g. `play_streaming.rs` + `play_streaming_gapless.rs`).
- `player/audio_thread/commands/dsd.rs` (split into `dsd_dop.rs`/`dsd_native.rs`
  given ~100-120 lines each already at the source level).
- `player/audio_thread/commands/transport.rs` (~130 lines) — Pause/Stop/
  SetVolume/ReinitDevice/ReleaseDevice (small arms).
- `player/audio_thread/commands/resume.rs` (~150 lines) — Resume alone.
- `player/audio_thread/commands/seek.rs` (~240 lines → likely still needs a
  second split, e.g. `seek.rs` + `seek_dsd.rs` for the DSD-specific seek path).
- `player/audio_thread/commands/gapless.rs` (~520 lines → split further, e.g.
  `gapless_prepare.rs` + `gapless_apply.rs`).
- `player/api/fetch.rs` — `prefetch_into_cache`, `is_track_cached`,
  `clear_audio_cache`, `fetch_for_gapless`, `fetch_for_external_stream`,
  `cmaf_stream_segments` (likely 2 files given ~500+ lines total).
- `player/api/play.rs` — `play_track`, `play_data`/`apply_play_data`,
  `play_next`, DSD file playback (`play_dsd_file`, `prepare_dsd_gapless_wav`,
  `play_next_dsd`, `is_dsd_direct_active`).
- `player/api/streaming.rs` — `play_streaming`/`apply_play_streaming`,
  `play_streaming_dynamic`/`apply_play_streaming_dynamic`, `download_audio`.
- `player/api/transport.rs` — `pause`/`resume`/`stop`/`set_volume`/`seek`/
  `reinit_device`/`release_device`/`reload_settings`/`get_state`/
  `get_playback_event`.
- `player/tests/*.rs` — split the existing `mod tests` by the same clusters
  (device/recreate tests, DSD error-report tests, content-type tests).

## Re-export surface
`player/mod.rs` stays the single import path: `use qbz_player::player::Player`
(and `SharedState`, `PlaybackEvent`, `PlaybackState`) must keep resolving
identically. All new submodules are private (`mod device; mod audio_thread;
mod api;` with no `pub`) except where a type is genuinely used by sibling
crates — check current `pub`/`pub(crate)` visibility on each item before
moving it, since some getters are `pub(crate)` (e.g. `begin_play`,
`is_current_play`) and used from outside this file within the crate.

## Tricky coupling / watch out
- The audio thread closure captures `thread_settings`, `thread_viz_tap`,
  `thread_diagnostic`, `analyzer_tx`/`analyzer_enabled`, `loudness_cache`, and
  the `wrap_source`/`init_device`/`is_device_valid` inline closures — every
  match arm calls into these. The `ThreadCtx` extraction must own or borrow
  all of them without changing drop order (the loudness analyzer thread and
  the stream must outlive the loop).
- `SharedState` is `Clone` and shared between the constructing thread and the
  audio thread (`thread_state`) — do not accidentally split its impl in a way
  that requires two lock acquisitions where one existed (check for methods
  that read multiple fields under one lock today).
- Gapless (`PlayNext`/`PlayNextDsdDop`/`GaplessPending`) shares state with the
  `Play`/DSD arms (`gapless_ready`, `gapless_next_track_id`) — keep these
  co-located enough that renames don't create silent behavior drift.
- `play_gen`/`begin_play`/`is_current_play` (the #583/#591 generation counter)
  is read from both the command-sender side (`api/`) and the thread side
  (`audio_thread/`) — must remain the exact same `SharedState` field.
- Backend-specific branches (ALSA direct, CoreAudio exclusive, DAC passthrough)
  are interleaved throughout `Play`/`PlayStreaming`/`Resume`/`Seek` — don't
  split by backend, split by command, or you'll fragment a single
  code path's backend handling across files.

## What to verify after the real split
- `cargo build -p qbz-player` and `cargo test -p qbz-player` (all listed unit
  tests, especially `dsd_error_report_*`, `*_reuses_stream`/`*_rebuilds`,
  `current_position_ms_is_a_pure_anchor_derivation`).
- Manual/smoke playback test via the `run` skill if available: play a track,
  pause/resume, seek, gapless transition, and a local DSD file if a fixture
  exists — the actual audio thread behavior is not exercised by unit tests.
- Grep every crate for `qbz_player::player::` and `qbz_player::Player` imports
  to confirm the public surface is unchanged.
