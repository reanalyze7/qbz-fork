# crates/qbz-dsd/src/native.rs (192 lines)

Native DSD_U32 packing for ALSA: streaming demux -> bit-reverse -> 4-byte
word packing, exposed as an `Iterator<Item = i32>`.

## Proposed split

- `native/mod.rs` (~110 lines) — re-export surface: `native_u32_rate`,
  `NATIVE_DSD_SILENCE_U32`, `NativeDsdStream` struct + `new`/`io_error`/
  `rate`/`dsd_rate`/`total_frames`/`pack_word`, and the `Iterator` impl.
- `native/refill.rs` (~55 lines) — the `refill` method body (the trickiest,
  most stateful part: carry-over bytes, bit-reversal, EOF padding, word
  packing) as a free fn taking `&mut NativeDsdStream`, or kept as an
  `impl NativeDsdStream` block in its own file.
- `native/tests.rs` (~20 lines) — existing test module.

## Tricky coupling

- `refill` mutates `self.carry`, `self.buf`, `self.idx`, `self.done`,
  `self.io_error` — all private fields on `NativeDsdStream`; splitting the
  impl block across files works fine in Rust as long as fields stay
  private-to-crate/module (same file tree). No functional risk.
- Depends on `crate::demux::{DsdDemuxer, DsdError}` and
  `crate::dsd2pcm::bit_reverse` — unchanged import paths.

## Verify after split

`cargo build -p qbz-dsd`, `cargo test -p qbz-dsd native::`.
