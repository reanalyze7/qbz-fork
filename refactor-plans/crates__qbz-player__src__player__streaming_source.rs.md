# crates/qbz-player/src/player/streaming_source.rs (1206 lines)

## Summary
Buffered/incremental streaming audio source: `BufferedMediaSource` (sync
`Read+Seek` wrapper over an async HTTP download, `Mutex`+`Condvar`
synchronized), `BufferWriter` (the async-side chunk pusher),
`IncrementalStreamingSource` (a rodio `Source` decoding symphonia packets
on-demand from the buffer), and `InMemorySource`/`InMemoryMediaSource` (a
native-seek decoder for fully-downloaded audio). Largest file in this
gap-fill batch. ~615 lines of logic, ~590 lines of tests.

## Proposed split (directory `streaming_source/`)
- `mod.rs` (~15 lines) — module declarations + re-exports.
- `config.rs` (~95 lines) — `StreamingConfig`, `from_seconds`,
  `fast_start`, `from_speed_mbps`, `from_speed_mbps_with_cap`,
  `raw_initial_buffer_for_speed`, the `MAX_INITIAL_BUFFER_BYTES` static +
  `set_max_initial_buffer_bytes`/`max_initial_buffer_bytes`.
- `buffer.rs` (~95 lines) — `BufferState`, `BufferedMediaSource` struct +
  its `new`/`create_reader` constructors and read-only accessors
  (`is_complete`, `buffer_size`, `take_complete_data`, `get_buffered_data`,
  `progress`, `has_min_buffer`, `download_error`).
- `buffer_io.rs` (~110 lines) — `impl Read for BufferedMediaSource`, `impl
  Seek for BufferedMediaSource`, `impl MediaSource for BufferedMediaSource`
  (the actual blocking I/O trait impls — kept separate from the plain
  struct/accessors above since they're the highest-risk, most-subtle code
  in the file).
- `writer.rs` (~65 lines) — `BufferWriter` (`push_chunk`, `complete`,
  `error`, `buffer_size`).
- `incremental.rs` (~185 lines) — `IncrementalStreamingSource` (struct +
  `new`/`get_sample_rate`/`get_channels`/`buffered_source`/`seek_to`/
  `decode_more`) + its `Source`/`Iterator` impls. Still over 130 lines on
  its own — `decode_more`'s WouldBlock/stall-tracking logic is one
  cohesive state machine that resists further splitting without hurting
  readability; flag as an accepted exception, or split `decode_more` itself
  into a private helper module if a stricter cut is required.
- `in_memory.rs` (~150 lines) — `InMemoryMediaSource`,
  `InMemorySource` (struct + `new`/`seek_to`/`decode_more`) + its
  `Source`/`Iterator` impls. Same near-duplicate `decode_more` shape as
  `incremental.rs` — flagged as a future dedup opportunity (a generic
  decode-loop shared by both) but NOT attempted here to keep this a
  behavior-preserving split.
- `tests/` — one file per production module (e.g. `tests_config.rs` for the
  buffer-size-ladder tests, `tests_buffer.rs` for the read/write/seek/error
  tests) — ~590 lines total, the single biggest lever for hitting budget.

## Re-export surface
`mod.rs` (i.e. `crate::player::streaming_source`) re-exports
`StreamingConfig`, `BufferedMediaSource`, `BufferWriter`,
`IncrementalStreamingSource`, `InMemorySource` — the five public types
consumed elsewhere in `qbz-player` (grep `streaming_source::` to confirm the
exact call sites before finalizing).

## Coupling / watch-outs
- `BufferedMediaSource`/`BufferWriter` share `Arc<(Mutex<BufferState>,
  Condvar)>` — `buffer.rs` and `writer.rs` both need this type; keep
  `BufferState`'s field visibility `pub(super)` (or keep the struct
  declaration in one file both submodules can `use super::BufferState`
  from).
- `take_complete_data`'s doc comment explains a REAL regression (an earlier
  `mem::take` attempt broke playback because the `Source` impl reads from
  `state.data` concurrently) — this comment MUST survive the split
  verbatim; it documents a non-obvious constraint a future editor could
  easily violate again.
- `IncrementalStreamingSource::decode_more`'s stall-tracking
  (`self.stalled`, `qbz_audio::network_throttle::state().record_underrun()`)
  is tied to issue #591 — keep that doc comment intact.
- `MAX_INITIAL_BUFFER_BYTES` is a process-wide atomic set once at startup
  from the host's memory profile (issue #331) — must stay a single
  `static` in `config.rs`, not duplicated.

## Verify after split
`cargo test -p qbz-player player::streaming_source::` (all ~15 existing
tests green, including the threaded `test_blocking_read`); `cargo build -p
qbz-player`; manual smoke-test: play a streamed (uncached) Hi-Res track end
to end and confirm no stutter/hiccup at the buffer-fill boundary.
